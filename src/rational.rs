use std::ops::{Div, Mul};

use num_traits::{Num, NumCast, PrimInt};

#[derive(Copy, Clone, Debug)]
pub struct Rational {
	num: u32,
	den: u32
}

pub trait Scalar: Num + NumCast {}

impl Scalar for u64 {}

impl Scalar for i64 {}

impl Scalar for f64 {}

impl Rational {
	pub fn gcd<T: PrimInt>(mut a: T, mut b: T) -> T {
		if a.is_zero() {
			return b;
		}

		if b.is_zero() {
			return a;
		}

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

			b = b - a;
			b = b.unsigned_shr(b.trailing_zeros());
		}

		a.unsigned_shl(trailing_zeroes)
	}

	pub fn new(num: u32, den: u32) -> Self {
		Self { num, den }
	}

	pub fn seconds() -> Self {
		Self { num: 1, den: 1 }
	}

	pub fn millis() -> Self {
		Self { num: 1, den: 1_000 }
	}

	pub fn micros() -> Self {
		Self { num: 1, den: 1_000_000 }
	}

	pub fn nanos() -> Self {
		Self { num: 1, den: 1_000_000_000 }
	}

	pub fn inverse(den: u32) -> Self {
		Self { num: 1, den }
	}

	pub fn invert(&self) -> Self {
		Self { num: self.den, den: self.num }
	}

	pub fn reduce(&mut self) -> &mut Self {
		let gcd = Self::gcd(self.num, self.den);

		if gcd != 0 {
			self.num /= gcd;
			self.den /= gcd;
		}

		self
	}

	pub fn rescale<T: Scalar>(&self, value: T, base: Rational) -> T {
		base * (self.invert() * value)
	}

	pub fn parts(&self) -> (u32, u32) {
		(self.num, self.den)
	}
}

impl Default for Rational {
	fn default() -> Self {
		Self { num: 0, den: 1 }
	}
}

impl<T: Scalar> Mul<T> for Rational {
	type Output = T;

	fn mul(self, rhs: T) -> T {
		T::from(self.num).unwrap() * rhs / T::from(self.den).unwrap()
	}
}

fn maybe_reduce(num: &mut u64, den: &mut u64) {
	if *num <= u32::MAX as u64 && *den <= u32::MAX as u64 {
		return;
	}

	let gcd = Rational::gcd(*num, *den);

	if gcd != 0 {
		*num /= gcd;
		*den /= gcd;
	}

	if *num > u32::MAX as u64 || *den > u32::MAX as u64 {
		panic!(
			"Failed to reduce rational to within u32 bounds: num = {}, den = {}",
			num, den
		);
	}
}

impl Mul for Rational {
	type Output = Rational;

	fn mul(self, rhs: Self) -> Self {
		let mut num = self.num as u64 * rhs.num as u64;
		let mut den = self.den as u64 * rhs.den as u64;

		maybe_reduce(&mut num, &mut den);

		Self { num: num as u32, den: den as u32 }
	}
}

impl Div for Rational {
	type Output = Rational;

	fn div(self, rhs: Self) -> Self {
		let mut num = self.num as u64 * rhs.den as u64;
		let mut den = self.den as u64 * rhs.num as u64;

		maybe_reduce(&mut num, &mut den);

		Self { num: num as u32, den: den as u32 }
	}
}
