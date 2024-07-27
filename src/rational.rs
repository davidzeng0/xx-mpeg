use std::ops::{Div, Mul};

use num_traits::{Num, NumCast, PrimInt};

#[derive(Copy, Clone, Debug)]
pub struct Rational {
	pub num: u32,
	pub den: u32
}

mod private {
	use super::*;

	pub trait Scalar: Num + NumCast {}
}

use self::private::*;

impl<T: Num + NumCast> Scalar for T {}

impl Rational {
	pub fn gcd<T>(mut a: T, mut b: T) -> T
	where
		T: PrimInt
	{
		if a.is_zero() {
			return b;
		}

		if b.is_zero() {
			return a;
		}

		#[allow(clippy::needless_late_init)]
		let trailing_zeroes = {
			let tz_a;
			let tz_b;

			tz_a = a.trailing_zeros();
			a = a.unsigned_shr(tz_a);

			tz_b = b.trailing_zeros();
			b = b.unsigned_shr(tz_b);

			tz_a.min(tz_b)
		};

		while a != b {
			if a > b {
				std::mem::swap(&mut a, &mut b);
			}

			#[allow(clippy::arithmetic_side_effects)]
			(b = b - a);
			b = b.unsigned_shr(b.trailing_zeros());
		}

		a.unsigned_shl(trailing_zeroes)
	}

	#[must_use]
	pub const fn new(num: u32, den: u32) -> Self {
		Self { num, den }
	}

	#[must_use]
	pub const fn seconds() -> Self {
		Self { num: 1, den: 1 }
	}

	#[must_use]
	pub const fn millis() -> Self {
		Self { num: 1, den: 1_000 }
	}

	#[must_use]
	pub const fn micros() -> Self {
		Self { num: 1, den: 1_000_000 }
	}

	#[must_use]
	pub const fn nanos() -> Self {
		Self { num: 1, den: 1_000_000_000 }
	}

	#[must_use]
	pub const fn inverse(den: u32) -> Self {
		Self { num: 1, den }
	}

	#[must_use]
	pub const fn invert(self) -> Self {
		Self { num: self.den, den: self.num }
	}

	#[must_use]
	pub fn reduce(mut self) -> Self {
		let gcd = Self::gcd(self.num, self.den);

		#[allow(clippy::arithmetic_side_effects)]
		if gcd != 0 {
			self.num /= gcd;
			self.den /= gcd;
		}

		self
	}

	#[allow(clippy::arithmetic_side_effects)]
	pub fn rescale<T>(self, value: T, base: Self) -> T
	where
		T: Scalar
	{
		base * (self.invert() * value)
	}
}

impl Default for Rational {
	fn default() -> Self {
		Self { num: 0, den: 1 }
	}
}

impl<T: Scalar> Mul<T> for Rational {
	type Output = T;

	#[allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
	fn mul(self, rhs: T) -> T {
		T::from(self.num).unwrap() * rhs / T::from(self.den).unwrap()
	}
}

/// # Panics
/// if the ratio cannot be reduced into u32 pairs
fn maybe_reduce(mut num: u64, mut den: u64) -> (u32, u32) {
	if let (Ok(num), Ok(den)) = (num.try_into(), den.try_into()) {
		return (num, den);
	}

	let gcd = Rational::gcd(num, den);

	#[allow(clippy::arithmetic_side_effects)]
	if gcd != 0 {
		num /= gcd;
		den /= gcd;
	}

	#[allow(clippy::panic)]
	let (Ok(num), Ok(den)) = (num.try_into(), den.try_into()) else {
		panic!(
			"Failed to reduce rational to within u32 bounds: num = {}, den = {}",
			num, den
		);
	};

	(num, den)
}

impl Mul for Rational {
	type Output = Self;

	#[allow(clippy::arithmetic_side_effects)]
	fn mul(self, rhs: Self) -> Self {
		let num = self.num as u64 * rhs.num as u64;
		let den = self.den as u64 * rhs.den as u64;

		let (num, den) = maybe_reduce(num, den);

		Self { num, den }
	}
}

impl Div for Rational {
	type Output = Self;

	#[allow(clippy::arithmetic_side_effects)]
	fn div(self, rhs: Self) -> Self {
		let num = self.num as u64 * rhs.den as u64;
		let den = self.den as u64 * rhs.num as u64;

		let (num, den) = maybe_reduce(num, den);

		Self { num, den }
	}
}
