use std::marker::PhantomData;

use xx_core::{error::*, impls::UIntExtensions, opt::hint::*};
use xx_pulse::*;

use crate::Reader;

pub mod spec;

pub type ElementId = u64;

#[derive(Clone)]
pub struct Element {
	pub id: ElementId,
	pub size: u64,
	pub offset: u64,
	pub end: u64
}

impl Element {
	pub fn remaining(&self, position: u64) -> i64 {
		if self.end != UNKNOWN_END {
			self.end.saturating_signed_difference(position)
		} else {
			i64::MAX
		}
	}
}

pub struct MasterElement {
	pub element: Element,
	pub children: &'static [ElementId]
}

impl MasterElement {
	pub const ROOT: MasterElement = MasterElement {
		element: Element {
			id: Self::ROOT_ID,
			size: UNKNOWN_SIZE,
			offset: 0,
			end: UNKNOWN_END
		},

		children: &[]
	};
	/* note: this is not a real element */
	pub const ROOT_ID: ElementId = 1;

	pub fn is_child(&self, id: ElementId) -> bool {
		if self.element.id != Self::ROOT_ID {
			self.children.contains(&id)
		} else {
			true
		}
	}
}

#[async_trait]
pub trait Parse: Sized {
	const ID: ElementId;
	const NAME: &'static str;

	async fn parse<P: Parser>(parser: &mut P, element: &Element) -> Result<Self>;

	fn post_parse(&mut self) -> Result<()>;
}

#[async_trait]
pub trait MasterParse: Parse {
	const CHILDREN: &'static [ElementId];

	async fn handle_child<P: Parser>(&mut self, parser: &mut P, element: &Element) -> Result<()>;
}

pub const fn make_id(id: u64) -> u64 {
	id ^ (1 << id.ilog2())
}

#[allow(dead_code)]
pub enum VintKind {
	Unsigned,
	Signed,
	Id,
	Size
}

pub const UNKNOWN_SIZE: u64 = u64::MAX;
pub const UNKNOWN_END: u64 = 0;

#[async_fn]
pub async fn read_vint(reader: &mut Reader, kind: VintKind) -> Result<u64> {
	let first = reader.read_u8().await?;

	if unlikely(first == 0) {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"EBML vint first byte cannot be zero"
		));
	}

	let length = first.leading_zeros();
	let lead = 1 << (7 - length);
	let length_bits = length * 8;
	let mask = (lead << length_bits) - 1;

	let mut result = 0;

	result |= (first as u64 ^ lead) << length_bits;
	result |= reader.read_vint_be(length as usize).await?;

	match kind {
		VintKind::Unsigned => (),
		VintKind::Signed => result -= mask >> 1,
		VintKind::Id if result == 0 || result == mask => {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"EBML id was zero or the maximum value"
			));
		}

		VintKind::Size if result == mask => {
			result = UNKNOWN_SIZE;
		}

		_ => ()
	}

	Ok(result)
}

#[async_fn]
pub async fn skip_element(reader: &mut Reader, element: &Element) -> Result<()> {
	if unlikely(element.size == UNKNOWN_SIZE) {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Unable to skip unhandled element of unknown size and unknown type"
		));
	}

	let amount = element.remaining(reader.position()) as u64;

	reader.skip(amount).await
}

#[async_fn]
pub async fn next_element(reader: &mut Reader, master: &Element) -> Result<Option<Element>> {
	let offset = reader.position();
	let remaining = master.remaining(offset);

	if remaining < 2 {
		return Ok(None);
	}

	let id = match read_vint(reader, VintKind::Id).await {
		Ok(id) => id,
		Err(err) => {
			return if err.kind() == ErrorKind::UnexpectedEof &&
				reader.position() == offset &&
				master.end == UNKNOWN_END
			{
				Ok(None)
			} else {
				Err(err)
			};
		}
	};

	let size = read_vint(reader, VintKind::Id).await?;
	let offset = reader.position();

	let end = if size != UNKNOWN_SIZE {
		match offset.checked_add(size) {
			Some(end) if master.end == UNKNOWN_END || end <= master.end => end,
			_ => {
				return Err(Error::new(
					ErrorKind::InvalidData,
					"Element size overflowed"
				))
			}
		}
	} else if master.end == UNKNOWN_END {
		UNKNOWN_END
	} else {
		master.end
	};

	Ok(Some(Element { id, size, offset, end }))
}

#[async_fn]
pub async fn read_children<P: Parser, F: FnMut(&mut P, &Element) -> Result<()>>(
	parser: &mut P, master: &MasterElement, mut handle_child: F
) -> Result<()> {
	loop {
		let reader = parser.reader();
		let element = match next_element(reader, &master.element).await? {
			Some(element) => element,
			None => break Ok(())
		};

		if master.is_child(element.id) {
			handle_child(parser, &element)?;
		} else {
			skip_element(reader, &element).await?;
		}
	}
}

#[async_trait]
pub trait Parser: Sized {
	fn reader(&mut self) -> &mut Reader;

	async fn read_children<F: FnMut(&mut Self, &Element) -> Result<()>>(
		&mut self, master: &MasterElement, handle_child: F
	) -> Result<()> {
		read_children(self, master, handle_child).await
	}

	fn pre_parse<E: Parse>(&self, _elem: &Element, _phantom: PhantomData<E>) -> Result<()> {
		Ok(())
	}
}
