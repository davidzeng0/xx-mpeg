use enumflags2::BitFlags;
use xx_core::macros::{macro_each, paste};

use super::*;

macro_rules! define_vint {
	(($name:ident, $kind:expr)) => {
		#[repr(transparent)]
		#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
		pub struct $name(pub u64);

		#[asynchronous]
		impl Element for $name {
			async fn parse<R>(reader: &mut R, _: &ElemHdr) -> Result<Self>
			where
				R: EbmlReader
			{
				let value = read_vint(reader, $kind).await?;

				Ok(Self(value))
			}
		}
	};
}

macro_each!(
	define_vint,
	(VInt, VIntKind::Unsigned),
	(VSInt, VIntKind::Signed),
	(VIntId, VIntKind::Id)
);

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Unsigned(pub u64);

impl From<u64> for Unsigned {
	fn from(value: u64) -> Self {
		Self(value)
	}
}

impl From<Unsigned> for u64 {
	fn from(value: Unsigned) -> Self {
		value.0
	}
}

#[asynchronous]
impl Element for Unsigned {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		if header.size <= size_of::<Self>() as u64 {
			#[allow(clippy::cast_possible_truncation)]
			Ok(Self(reader.read_vint_be(header.size as usize).await?))
		} else {
			Err(FormatError::InvalidData("Invalid size for unsigned".into()).into())
		}
	}
}

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Signed(pub i64);

impl From<i64> for Signed {
	fn from(value: i64) -> Self {
		Self(value)
	}
}

impl From<Signed> for i64 {
	fn from(value: Signed) -> Self {
		value.0
	}
}

#[asynchronous]
impl Element for Signed {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value = Unsigned::parse(reader, header).await?;

		#[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
		let high_bits = u64::BITS - header.size as u32 * 8;

		#[allow(clippy::cast_possible_wrap)]
		let value = (value.0.wrapping_shl(high_bits) as i64).wrapping_shr(high_bits);

		Ok(Self(value))
	}
}

#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Float(pub f64);

impl From<f64> for Float {
	fn from(value: f64) -> Self {
		Self(value)
	}
}

impl From<Float> for f64 {
	fn from(value: Float) -> Self {
		value.0
	}
}

#[asynchronous]
impl Element for Float {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value;

		if header.size == 0 {
			value = 0.0;
		} else if header.size == size_of::<f32>() as u64 {
			value = reader.read_f32_be().await? as f64;
		} else if header.size == size_of::<f64>() as u64 {
			value = reader.read_f64_be().await?;
		} else {
			return Err(FormatError::InvalidData("Invalid size for float".into()).into());
		}

		Ok(Self(value))
	}
}

#[asynchronous]
impl Element for () {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		reader.skip(header.size).await?;

		Ok(())
	}
}

#[asynchronous]
impl Element for String {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		#[allow(clippy::unwrap_used)]
		reader
			.read_string(header.size.try_into().map_err(|_| ErrorKind::Overflow)?)
			.await
	}
}

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Bytes(pub Vec<u8>);

#[asynchronous]
impl Element for Bytes {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		#[allow(clippy::unwrap_used)]
		Ok(Self(
			reader
				.read_bytes(header.size.try_into().map_err(|_| ErrorKind::Overflow)?)
				.await?
		))
	}
}

macro_rules! impl_prim {
	($type:ty) => {
		#[asynchronous]
		impl Element for $type {
			async fn parse<R>(reader: &mut R, _: &ElemHdr) -> Result<Self>
			where
				R: EbmlReader
			{
				paste! { reader.[<read_ $type _be>]() }.await
			}
		}
	};
}

macro_rules! impl_byte {
	($type:ty) => {
		#[asynchronous]
		impl Element for $type {
			async fn parse<R>(reader: &mut R, _: &ElemHdr) -> Result<Self>
			where
				R: EbmlReader
			{
				paste! { reader.[<read_ $type>]() }.await
			}
		}
	};
}

macro_each!(impl_prim, u128, i128, u64, i64, u32, i32, u16, i16);
macro_each!(impl_byte, u8, i8);

#[asynchronous]
impl Element for bool {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value = Unsigned::parse(reader, header).await?;

		Ok(match value.0 {
			0 => false,
			1 => true,
			_ => return Err(EbmlError::InvalidVariant.into())
		})
	}
}

#[asynchronous]
impl<T: BitFlag<Numeric = u64>> Element for BitFlags<T> {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value = Unsigned::parse(reader, header).await?;

		BitFlags::from_bits(value.0).map_err(|_| EbmlError::InvalidVariant.into())
	}
}

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct NonZeroUnsigned(pub u64);

#[asynchronous]
impl Element for NonZeroUnsigned {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value = Unsigned::parse(reader, header).await?;

		if value.0 != 0 {
			Ok(Self(value.0))
		} else {
			Err(EbmlError::ExpectedNonZero.into())
		}
	}
}

impl From<u64> for NonZeroUnsigned {
	fn from(value: u64) -> Self {
		Self(value)
	}
}

impl From<NonZeroUnsigned> for u64 {
	fn from(value: NonZeroUnsigned) -> Self {
		value.0
	}
}

#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct PositiveFloat(pub f64);

impl From<f64> for PositiveFloat {
	fn from(value: f64) -> Self {
		Self(value)
	}
}

impl From<PositiveFloat> for f64 {
	fn from(value: PositiveFloat) -> Self {
		value.0
	}
}

#[asynchronous]
impl Element for PositiveFloat {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value = Float::parse(reader, header).await?;

		if value.0 > 0.0 {
			Ok(Self(value.0))
		} else {
			Err(EbmlError::ExpectedNonZero.into())
		}
	}
}

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct NonEmptyString(pub String);

impl<T: Into<String>> From<T> for NonEmptyString {
	fn from(value: T) -> Self {
		Self(value.into())
	}
}

#[asynchronous]
impl Element for NonEmptyString {
	async fn parse<R>(reader: &mut R, header: &ElemHdr) -> Result<Self>
	where
		R: EbmlReader
	{
		let value = String::parse(reader, header).await?;

		if !value.is_empty() {
			Ok(Self(value))
		} else {
			Err(EbmlError::ExpectedNonZero.into())
		}
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Date(pub Unsigned);
}

ebml_define! {
	#[allow(dead_code)]
	pub struct UUID(pub u128);
}
