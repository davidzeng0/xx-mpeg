use std::num::NonZeroU64;

use super::*;

#[errors]
pub enum EbmlError {
	#[display("EBML vint first byte cannot be zero")]
	#[kind = ErrorKind::InvalidData]
	InvalidVint,

	#[display("EBML id was zero or the maximum value")]
	#[kind = ErrorKind::InvalidData]
	InvalidId,

	#[display("Found a duplicate element")]
	#[kind = ErrorKind::InvalidData]
	DuplicateElement,

	#[display("Required element is missing")]
	#[kind = ErrorKind::InvalidData]
	MissingElement,

	#[display("Expected non zero data")]
	#[kind = ErrorKind::InvalidData]
	ExpectedNonZero,

	#[display("Invalid enum variant")]
	#[kind = ErrorKind::InvalidData]
	InvalidVariant
}

pub const fn make_id(id: u64) -> u64 {
	id ^ (1 << id.ilog2())
}

#[asynchronous]
#[inline(always)]
#[allow(clippy::arithmetic_side_effects)]
pub async fn read_vint(reader: &mut Reader, kind: VIntKind) -> Result<u64> {
	let first = reader.read_u8().await?;

	if unlikely(first == 0) {
		return Err(EbmlError::InvalidVint.into());
	}

	let length = first.leading_zeros();
	let lead = 1 << (7 - length);
	let length_bits = length * 8;
	let mask = (lead << length_bits) - 1;

	let mut result = 0;

	result |= (first as u64 ^ lead) << length_bits;
	result |= reader.read_vint_be(length as usize).await?;

	match kind {
		VIntKind::Unsigned => (),
		VIntKind::Signed => result = result.wrapping_sub(mask >> 1),
		VIntKind::Id if result == 0 || result == mask => return Err(EbmlError::InvalidId.into()),
		VIntKind::Size if result == mask => result = UNKNOWN_SIZE,
		_ => ()
	}

	Ok(result)
}

#[derive(Copy, Clone)]
pub struct ElemHdr {
	pub id: EbmlId,
	pub size: u64,
	pub offset: u64,
	pub end: Option<NonZeroU64>
}

impl ElemHdr {
	pub const fn known_end(&self) -> bool {
		self.end.is_some()
	}

	pub fn remaining(&self, position: u64) -> Option<u64> {
		self.end.map(|end| end.get().saturating_sub(position))
	}

	pub fn has(&self, position: u64, requested: u64) -> bool {
		match self.remaining(position) {
			Some(remaining) => remaining >= requested,
			None => true
		}
	}
}

#[derive(Copy, Clone)]
pub struct MasterElemHdr {
	pub element: ElemHdr,
	pub children: &'static [EbmlId]
}

#[asynchronous]
impl MasterElemHdr {
	pub async fn next_element(&self, reader: &mut Reader) -> Result<Option<ElemHdr>> {
		let position = reader.position();

		if !self.element.has(position, 2) {
			return Ok(None);
		}

		let id = match read_vint(reader, VIntKind::Id).await {
			Ok(id) => id,
			Err(err) => {
				return if err == ErrorKind::UnexpectedEof &&
					reader.position() == position &&
					!self.element.known_end()
				{
					Ok(None)
				} else {
					Err(err)
				}
			}
		};

		let size = read_vint(reader, VIntKind::Size).await?;
		let offset = reader.position();

		let end = if size == UNKNOWN_SIZE {
			self.element.end
		} else {
			if !self.element.has(offset, size) {
				return Err(FormatError::ReadOverflow.into());
			}

			let sum = offset.checked_add(size).ok_or(ErrorKind::Overflow)?;

			#[allow(clippy::unwrap_used)]
			Some(NonZeroU64::new(sum).unwrap())
		};

		Ok(Some(ElemHdr { id, size, offset, end }))
	}

	pub fn is_child(&self, id: EbmlId) -> bool {
		self.children.contains(&id)
	}

	pub const fn root<T>() -> Self
	where
		T: MasterElement
	{
		Self {
			element: ElemHdr { id: 0, size: UNKNOWN_SIZE, offset: 0, end: None },
			children: T::CHILDREN
		}
	}

	pub const fn default() -> Self {
		Self {
			element: ElemHdr { id: 0, size: UNKNOWN_SIZE, offset: 0, end: None },
			children: &[]
		}
	}
}

impl Default for MasterElemHdr {
	fn default() -> Self {
		Self::default()
	}
}

#[asynchronous]
pub async fn default_read_children<R, F>(
	reader: &mut R, master: &MasterElemHdr, mut handle_child: F
) -> Result<()>
where
	R: EbmlReader + ?Sized,
	F: AsyncFnMut(&mut R, &ElemHdr) -> Result<bool>
{
	loop {
		let element = match master.next_element(&mut *reader).await? {
			Some(element) => element,
			None => break Ok(())
		};

		if master.is_child(element.id) {
			handle_child.call_mut((reader, &element)).await?;
		} else {
			reader.skip_element(&element).await?;
		}
	}
}

#[asynchronous]
pub async fn default_skip_element<R>(reader: &mut R, element: &ElemHdr) -> Result<()>
where
	R: EbmlReader + ?Sized
{
	if let Some(remaining) = element.remaining(reader.position()) {
		reader.skip(remaining).await
	} else {
		let msg = "Cannot skip an element of unknown size and unknown type";

		Err(FormatError::InvalidData(msg.into()).into())
	}
}

#[asynchronous]
pub trait EbmlReader: DerefMut<Target = Reader> {
	async fn read_children<F>(&mut self, master: &MasterElemHdr, handle_child: F) -> Result<()>
	where
		F: AsyncFnMut(&mut Self, &ElemHdr) -> Result<bool>
	{
		default_read_children(self, master, handle_child).await
	}

	async fn skip_element(&mut self, element: &ElemHdr) -> Result<()> {
		default_skip_element(self, element).await
	}

	fn trace_element(&self, _name: &str, _elem: &ElemHdr) {}
}

#[asynchronous]
pub trait Element: Sized {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader;
}

#[asynchronous]
pub trait MasterElement: Element {
	const CHILDREN: &'static [EbmlId];

	async fn handle_child<R>(&mut self, reader: &mut R, header: &ElemHdr) -> Result<bool>
	where
		R: EbmlReader;
}
