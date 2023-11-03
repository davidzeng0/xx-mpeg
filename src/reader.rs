use std::{io::SeekFrom, mem::size_of};

use xx_core::{async_std::io::*, error::*, impls::UIntExtensions, opt::hint::*};
use xx_pulse::*;

use crate::resource::*;

pub const DEFAULT_SEEK_THRESHOLD: u64 = 512 * 1024;

pub struct Reader {
	stream: TypedReader<BufReader<Stream>>,
	position: u64,
	seek_threshold: u64,
	peeking: Option<u64>,
	seekable: bool
}

macro_rules! read_num_type_endian {
	($type: ty, $endian_type: ident) => {
		paste::paste! {
			#[inline(always)]
			#[async_fn]
			pub async fn [<read_ $endian_type>](&mut self) -> Result<$type> {
				self.maybe_peeking_reserve_space(size_of::<$type>()).await?;

				let result = self.stream.[<read_ $endian_type _or_err>]().await?;

				self.position += size_of::<$type>() as u64;

				Ok(result)
			}
		}
	};
}

macro_rules! read_num_type {
	($type: ty) => {
		paste::paste! {
			read_num_type_endian!($type, [<$type _le>]);
			read_num_type_endian!($type, [<$type _be>]);
		}
	};
}

macro_rules! read_int {
	($bits: literal) => {
		paste::paste! {
			read_num_type!([<i $bits>]);
			read_num_type!([<u $bits>]);
		}
	};
}

#[async_fn]
impl Reader {
	read_num_type_endian!(i8, i8);

	read_num_type_endian!(u8, u8);

	read_int!(16);

	read_int!(32);

	read_int!(64);

	read_int!(128);

	read_num_type!(f32);

	read_num_type!(f64);

	#[inline(never)]
	async fn do_reserve_space(&mut self, space: usize) -> Result<()> {
		loop {
			let spare = self.stream.spare_capacity() + self.stream.buffer().len();

			if spare < space {
				return Err(Error::new(ErrorKind::InvalidInput, "Peek buffer exhaused"));
			}

			let filled = self.stream.fill_amount(spare).await?;

			if unlikely(filled == 0) {
				return Err(Error::Simple(ErrorKind::UnexpectedEof));
			}

			if self.stream.buffer().len() >= space {
				break;
			}
		}

		Ok(())
	}

	#[inline(always)]
	async fn maybe_peeking_reserve_space(&mut self, space: usize) -> Result<()> {
		if self.peeking.is_none() || self.stream.buffer().len() >= space {
			Ok(())
		} else {
			self.do_reserve_space(space).await
		}
	}

	pub fn new(stream: Stream) -> Self {
		let seek_threshold = stream.suggested_seek_threshold();

		Self {
			stream: TypedReader::new(BufReader::new(stream)),
			position: 0,
			seek_threshold,
			peeking: None,
			seekable: true
		}
	}

	/// If doing a relative seek forwards on a stream with
	/// an expensive seek operation
	///
	/// Prefer to read until that offset rather than seek if
	/// the difference <= threshold
	pub fn set_seek_threshold(&mut self, threshold: u64) {
		self.seek_threshold = threshold;
	}

	async fn consume(&mut self, mut left: usize) -> Result<()> {
		let amount = left;

		loop {
			let available = self.stream.buffer().len();

			if left > available {
				left -= available;

				self.stream.discard();

				if self.stream.fill().await? == 0 {
					return Err(unexpected_end_of_stream());
				}
			} else {
				unsafe {
					self.stream.consume_unchecked(left);
				}

				break;
			}
		}

		self.position += amount as u64;

		Ok(())
	}

	async fn seek_relative(&mut self, rel: i64) -> Result<()> {
		if rel >= 0 && (rel as u64 <= self.seek_threshold || !self.seekable) {
			self.consume(rel as usize).await?;
		} else {
			/* overflow is checked by calling function */
			self.position = self.position.wrapping_add_signed(rel);

			if rel < 0 && -rel as usize <= self.stream.position() {
				unsafe {
					self.stream.consume_unchecked(rel as usize);
				}
			} else {
				self.position = self.stream.seek(SeekFrom::Start(self.position)).await?;
			}
		}

		Ok(())
	}

	pub async fn seek(&mut self, seek: SeekFrom) -> Result<()> {
		let ((rel, overflow), abs) = match seek {
			SeekFrom::Current(pos) => {
				((pos, false), self.position.checked_add_signed(pos).unwrap())
			}

			SeekFrom::Start(pos) => (pos.overflowing_signed_difference(self.position), pos),

			SeekFrom::End(pos) => {
				let pos = self
					.stream
					.stream_len()
					.await?
					.checked_add_signed(pos)
					.unwrap();
				(pos.overflowing_signed_difference(self.position), pos)
			}
		};

		if !overflow {
			self.seek_relative(rel).await?;
		} else {
			self.position = self.stream.seek(SeekFrom::Start(abs)).await?;
		}

		if unlikely(self.position != abs) {
			self.seekable = false;

			/* on non seekable streams, permit seeking to zero, but not past the
			 * requested position */
			if let Some(amt) = abs.checked_sub(self.position) {
				self.consume(amt as usize).await?;
			} else {
				return Err(Error::new(
					ErrorKind::Other,
					format!("Invalid seek: requested {}, got {}", abs, self.position)
				));
			}
		}

		Ok(())
	}

	pub async fn skip(&mut self, amount: u64) -> Result<()> {
		self.seek(SeekFrom::Current(amount.try_into().unwrap()))
			.await
	}

	pub async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
		self.stream.read_exact_or_err(buf).await?;
		self.position += buf.len() as u64;

		Ok(())
	}

	pub fn position(&mut self) -> u64 {
		self.position
	}

	pub async fn set_peeking(&mut self, peeking: bool) {
		if peeking == self.peeking.is_some() {
			return;
		}

		if peeking {
			self.peeking = Some(self.position);

			if self.stream.position() != 0 {
				self.stream.move_data_to_beginning();
			}
		} else {
			let rel = self.peeking.take().unwrap().wrapping_sub(self.position);

			/* seek should be within our buffer, no errors should occur */
			self.seek_relative(rel as i64).await.unwrap();
		}
	}

	#[inline(never)]
	async fn read_bytes_oob_slow<const N: usize>(&mut self, size: usize) -> Result<[u8; N]> {
		let mut bytes = [0u8; N];

		if self.stream.read_exact(&mut bytes[0..size]).await? == size {
			self.position += size as u64;

			Ok(bytes)
		} else {
			Err(unexpected_end_of_stream())
		}
	}

	#[inline(always)]
	async fn read_bytes_oob<const N: usize>(&mut self, size: usize) -> Result<[u8; N]> {
		if self.stream.buffer().len() >= N {
			let mut bytes = [0u8; N];

			unsafe {
				read_into_slice(&mut bytes, self.stream.buffer().get_unchecked(0..N));

				self.stream.consume_unchecked(size);
				self.position += size as u64;
			}

			Ok(bytes)
		} else {
			self.read_bytes_oob_slow(size).await
		}
	}

	#[inline(always)]
	async fn read_vint_fast(&mut self, size: usize, le: bool) -> Result<u64> {
		let unsigned_size = size_of::<u64>();

		if unlikely(size == 0 || size > unsigned_size) {
			return if size == 0 {
				Ok(0)
			} else {
				Err(Error::new(ErrorKind::InvalidData, "Invalid vint size"))
			};
		}

		let bytes = self.read_bytes_oob::<{ size_of::<u64>() }>(size).await?;

		let value = if le {
			u64::from_le_bytes(bytes)
		} else {
			u64::from_be_bytes(bytes)
		};

		let shift = (unsigned_size - size) as u32 * 8;

		Ok(if le {
			let mask = u64::MAX.wrapping_shr(shift);

			/* LE 0ABC: CBA -> ...ABC -> shave top */
			(value & mask) as u64
		} else {
			/* BE 0ABC: ABC -> ABC... -> shave bottom */
			value.wrapping_shr(shift) as u64
		})
	}

	#[inline(always)]
	pub async fn read_vint_le(&mut self, size: usize) -> Result<u64> {
		self.read_vint_fast(size, true).await
	}

	#[inline(always)]
	pub async fn read_vint_be(&mut self, size: usize) -> Result<u64> {
		self.read_vint_fast(size, false).await
	}

	async fn read_vfloat(&mut self, size: usize, le: bool) -> Result<f64> {
		if size == size_of::<f32>() {
			Ok(if le {
				self.read_f32_le().await?
			} else {
				self.read_f32_be().await?
			} as f64)
		} else if size == size_of::<f64>() {
			Ok(if le {
				self.read_f64_le().await?
			} else {
				self.read_f64_be().await?
			})
		} else {
			Err(Error::new(
				ErrorKind::InvalidData,
				format!("Invalid variable sized float of size {}", size)
			))
		}
	}

	pub async fn read_vfloat_le(&mut self, size: usize) -> Result<f64> {
		self.read_vfloat(size, true).await
	}

	pub async fn read_vfloat_be(&mut self, size: usize) -> Result<f64> {
		self.read_vfloat(size, false).await
	}

	pub async fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>> {
		let mut buf = Vec::with_capacity(size);

		unsafe {
			buf.set_len(size);
		}

		self.read(&mut buf).await?;

		Ok(buf)
	}

	pub async fn read_string(&mut self, size: usize) -> Result<String> {
		let buf = self.read_bytes(size).await?;

		Ok(String::from_utf8(buf).map_err(|_| invalid_utf8_error())?)
	}

	pub async fn eof(&mut self) -> Result<bool> {
		if self.stream.buffer().len() > 0 {
			Ok(false)
		} else {
			Ok(self.stream.fill().await? != 0)
		}
	}
}
