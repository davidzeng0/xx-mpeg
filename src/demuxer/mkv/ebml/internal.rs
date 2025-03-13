use xx_core::macros::sealed_trait;

use super::*;

sealed_trait!();

pub trait FieldMeta: Sized + Sealed {
	type Element: Element;
	type Output;
	const MULTIPLE: bool;
}

pub trait FieldInit<T>: Sized + FieldMeta {
	fn insert(&mut self, value: T) -> Result<()>;

	fn get(self) -> Result<Self::Output>;

	fn get_or_default<F, D>(self, default: F) -> Self::Output
	where
		F: FnOnce() -> D,
		D: Into<Self::Output>;
}

impl<T: Element> Sealed for Option<T> {}

impl<T: Element> FieldMeta for Option<T> {
	type Element = T;
	type Output = T;

	const MULTIPLE: bool = false;
}

impl<T: Element> FieldInit<T> for Option<T> {
	fn insert(&mut self, value: T) -> Result<()> {
		if self.is_none() {
			*self = Some(value);

			Ok(())
		} else {
			Err(EbmlError::DuplicateElement.into())
		}
	}

	fn get(self) -> Result<Self::Output> {
		self.ok_or_else(|| EbmlError::MissingElement.into())
	}

	fn get_or_default<F, D>(self, default: F) -> Self::Output
	where
		F: FnOnce() -> D,
		D: Into<Self::Output>
	{
		self.unwrap_or_else(|| default().into())
	}
}

impl<T: Element> Sealed for Option<Option<T>> {}

impl<T: Element> FieldMeta for Option<Option<T>> {
	type Element = T;
	type Output = Option<T>;

	const MULTIPLE: bool = false;
}

impl<T: Element> FieldInit<T> for Option<Option<T>> {
	fn insert(&mut self, value: T) -> Result<()> {
		if self.is_none() {
			*self = Some(Some(value));

			Ok(())
		} else {
			Err(EbmlError::DuplicateElement.into())
		}
	}

	fn get(self) -> Result<Self::Output> {
		Ok(self.unwrap_or(None))
	}

	fn get_or_default<F, D>(self, default: F) -> Self::Output
	where
		F: FnOnce() -> D,
		D: Into<Self::Output>
	{
		self.unwrap_or_else(|| default().into())
	}
}

impl<T: Element> Sealed for Option<Vec<T>> {}

impl<T: Element> FieldMeta for Option<Vec<T>> {
	type Element = T;
	type Output = Vec<T>;

	const MULTIPLE: bool = true;
}

impl<T: Element> FieldInit<T> for Option<Vec<T>> {
	fn insert(&mut self, value: T) -> Result<()> {
		self.get_or_insert_with(|| Vec::new()).push(value);

		Ok(())
	}

	fn get(self) -> Result<Self::Output> {
		self.ok_or_else(|| EbmlError::MissingElement.into())
	}

	fn get_or_default<F, D>(self, default: F) -> Self::Output
	where
		F: FnOnce() -> D,
		D: Into<Self::Output>
	{
		self.unwrap_or_else(|| default().into())
	}
}

impl<T: Element> Sealed for Option<Option<Vec<T>>> {}

impl<T: Element> FieldMeta for Option<Option<Vec<T>>> {
	type Element = T;
	type Output = Vec<T>;

	const MULTIPLE: bool = true;
}

impl<T: Element> FieldInit<T> for Option<Option<Vec<T>>> {
	fn insert(&mut self, value: T) -> Result<()> {
		self.get_or_insert_with(|| None)
			.get_or_insert_with(|| Vec::new())
			.push(value);

		Ok(())
	}

	fn get(self) -> Result<Self::Output> {
		Ok(self.unwrap_or(None).unwrap_or_default())
	}

	fn get_or_default<F, D>(self, default: F) -> Self::Output
	where
		F: FnOnce() -> D,
		D: Into<Self::Output>
	{
		self.unwrap_or(None).unwrap_or_else(|| default().into())
	}
}

pub trait EnumRepr: Sealed {
	fn convert<E>(self) -> Option<E>
	where
		E: FromPrimitive;
}

impl Sealed for Unsigned {}

impl EnumRepr for Unsigned {
	fn convert<E>(self) -> Option<E>
	where
		E: FromPrimitive
	{
		E::from_u64(self.0)
	}
}

impl Sealed for Signed {}

impl EnumRepr for Signed {
	fn convert<E>(self) -> Option<E>
	where
		E: FromPrimitive
	{
		E::from_i64(self.0)
	}
}
