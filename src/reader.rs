#![allow(unreachable_pub)]

use std::io::SeekFrom;
use std::mem::size_of;

use xx_core::async_std::io::typed::BufReadTyped;
use xx_core::async_std::io::*;
use xx_core::coroutines::ops::AsyncFnOnce;
use xx_core::impls::UintExt;
use xx_core::macros::{macro_each, paste};
use xx_core::opt::hint::*;

use super::*;
use crate::resource::*;
use crate::FormatError;

macro_rules! read_num_type_endian {
	($type:ty, $endian_type:ident) => {
		paste! {
			#[asynchronous]
			#[inline]
			pub async fn [<read_ $endian_type>](&mut self) -> Result<$type> {
				self.do_read(
					size_of::<$type>(),
					#[inline]
					|this: &mut Self| async move { this.stream.[<read_ $endian_type>]().await }
				).await
			}
		}
	};
}

macro_rules! read_num_type {
	($type:ty) => {
		paste! {
			read_num_type_endian!($type, [<$type _le>]);
			read_num_type_endian!($type, [<$type _be>]);
		}
	};
}

macro_rules! read_int {
	($bits:literal) => {
		paste! {
			read_num_type!([<i $bits>]);
			read_num_type!([<u $bits>]);
		}
	};
}

#[errors]
pub enum ReaderError {
	#[display("Peek buffer exhausted")]
	PeekBufferExhausted
}

pub struct Reader {
	stream: BufReader<Stream>,
	position: u64,
	seek_threshold: u64,
	peeking: Option<u64>,
	seekable: bool
}

#[asynchronous]
impl Reader {
	read_num_type_endian!(i8, i8);

	read_num_type_endian!(u8, u8);

	macro_each!(read_int, 16, 32, 64, 128);

	macro_each!(read_num_type, f32, f64);

	#[cold]
	async fn reserve_space(&mut self, space: usize) -> Result<()> {
		loop {
			#[allow(clippy::arithmetic_side_effects)]
			let spare = self.stream.spare_capacity() + self.stream.buffer().len();

			if spare < space {
				return Err(ReaderError::PeekBufferExhausted.into());
			}

			let filled = self.stream.fill_amount(spare).await?;

			if unlikely(filled == 0) {
				return Err(ErrorKind::UnexpectedEof.into());
			}

			if self.stream.buffer().len() >= space {
				break;
			}
		}

		Ok(())
	}

	#[inline(always)]
	async fn do_read<T, F>(&mut self, len: usize, read: F) -> Result<T>
	where
		F: AsyncFnOnce(&mut Self) -> Result<T>
	{
		if unlikely(self.peeking.is_some() && self.stream.buffer().len() < len) {
			self.reserve_space(len).await?;
		}

		let value = read.call_once(self).await?;

		#[allow(clippy::arithmetic_side_effects)]
		(self.position += len as u64);

		Ok(value)
	}

	pub fn new(stream: Stream) -> Self {
		let seek_threshold = stream.suggested_seek_threshold();
		let seekable = stream.seekable();

		Self {
			stream: BufReader::new(stream),
			position: 0,
			seek_threshold,
			peeking: None,
			seekable
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

	/// Skip bytes without seeking
	async fn consume(&mut self, mut left: u64) -> Result<()> {
		let amount = left;

		loop {
			let available = self.stream.buffer().len();

			if left > available as u64 {
				#[allow(clippy::arithmetic_side_effects)]
				(left -= available as u64);

				self.stream.discard();

				if self.stream.fill().await? == 0 {
					return Err(ErrorKind::UnexpectedEof.into());
				}
			} else {
				#[allow(clippy::cast_possible_truncation)]
				self.stream.consume(left as usize);

				break;
			}
		}

		#[allow(clippy::arithmetic_side_effects)]
		(self.position += amount);

		Ok(())
	}

	async fn seek_relative(&mut self, rel: i64) -> Result<()> {
		#[allow(clippy::cast_sign_loss)]
		if rel >= 0 && (rel as u64 <= self.seek_threshold || !self.seekable) {
			self.consume(rel as u64).await?;
		} else {
			let new_pos = (self.stream.position() as u64).checked_add_signed(rel);

			if rel <= 0 && new_pos.is_some() {
				#[allow(clippy::cast_possible_truncation, clippy::arithmetic_side_effects)]
				self.stream.unconsume(-rel as usize);
				self.position = self.position.wrapping_add_signed(rel);
			} else {
				#[allow(clippy::unwrap_used)]
				let pos = self.position.checked_add_signed(rel).unwrap();

				self.position = self.stream.seek(SeekFrom::Start(pos)).await?;
			}
		}

		Ok(())
	}

	pub async fn seek(&mut self, seek: SeekFrom) -> Result<()> {
		#[allow(clippy::unwrap_used, unstable_name_collisions)]
		let (rel, abs) = match seek {
			SeekFrom::Current(pos) => (Some(pos), self.position.checked_add_signed(pos).unwrap()),

			SeekFrom::Start(pos) => (pos.checked_signed_diff(self.position), pos),

			SeekFrom::End(pos) => {
				let pos = self.len().await?.checked_add_signed(pos).unwrap();

				(pos.checked_signed_diff(self.position), pos)
			}
		};

		if rel.is_some_and(|rel| rel >= 0 || self.seekable) {
			#[allow(clippy::unwrap_used)]
			self.seek_relative(rel.unwrap()).await?;
		} else {
			self.position = self.stream.seek(SeekFrom::Start(abs)).await?;
		};

		if unlikely(self.position != abs) {
			self.seekable = false;

			/* on non seekable streams, permit seeking to zero, but not past the
			 * requested position */
			if let Some(amt) = abs.checked_sub(self.position) {
				self.consume(amt).await?;
			} else {
				return Err(FormatError::InvalidSeek(abs, self.position).into());
			}
		}

		Ok(())
	}

	pub async fn skip(&mut self, amount: u64) -> Result<()> {
		if let Ok(amount) = i64::try_from(amount) {
			return self.seek(SeekFrom::Current(amount)).await;
		}

		self.position = self.stream.skip_amount(amount).await?;

		Ok(())
	}

	pub async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
		self.do_read(buf.len(), |this: &mut Self| async move {
			this.stream.read_fully(buf).await
		})
		.await?;

		Ok(())
	}

	pub async fn read_partial(&mut self, buf: &mut [u8]) -> Result<usize> {
		if unlikely(self.peeking.is_some() && self.stream.buffer().is_empty()) {
			let spare = self.stream.spare_capacity();

			if spare == 0 {
				return Err(ReaderError::PeekBufferExhausted.into());
			}

			self.reserve_space(spare).await?;
		}

		let read = self.stream.read(buf).await?;

		#[allow(clippy::arithmetic_side_effects)]
		(self.position += read as u64);

		Ok(read)
	}

	#[inline]
	pub async fn read_vint_le(&mut self, size: usize) -> Result<u64> {
		self.do_read(
			size,
			#[inline]
			|this: &mut Self| async move { this.stream.read_vint_u64_le(size).await }
		)
		.await
	}

	#[inline]
	pub async fn read_vint_be(&mut self, size: usize) -> Result<u64> {
		self.do_read(
			size,
			#[inline]
			|this: &mut Self| async move { this.stream.read_vint_u64_be(size).await }
		)
		.await
	}

	#[inline]
	pub async fn read_vfloat_le(&mut self, size: usize) -> Result<f64> {
		self.do_read(
			size,
			#[inline]
			|this: &mut Self| async move { this.stream.read_vfloat_le(size).await }
		)
		.await
	}

	#[inline]
	pub async fn read_vfloat_be(&mut self, size: usize) -> Result<f64> {
		self.do_read(
			size,
			#[inline]
			|this: &mut Self| async move { this.stream.read_vfloat_be(size).await }
		)
		.await
	}

	pub async fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>> {
		const READ_LIMIT: usize = 1024 * 1024;

		self.do_read(size, |this: &mut Self| async move {
			if size <= READ_LIMIT {
				let mut buf = vec![0; size];

				this.stream.read_fully(&mut buf).await?;

				return Ok(buf);
			}

			let mut buf = vec![0; READ_LIMIT];
			let mut offset = 0;

			while offset < size {
				if offset == buf.capacity() {
					#[allow(clippy::arithmetic_side_effects)]
					let addl = (size - offset).min(READ_LIMIT);

					buf.reserve_exact(addl);
					buf.resize(addl, 0);
				}

				this.stream.read_fully(&mut buf[offset..]).await?;
				offset = buf.len();
			}

			Ok(buf)
		})
		.await
	}

	pub async fn read_string(&mut self, size: usize) -> Result<String> {
		let buf = self.read_bytes(size).await?;

		String::from_utf8(buf).map_err(Into::into)
	}

	pub const fn position(&self) -> u64 {
		self.position
	}

	pub const fn seekable(&self) -> bool {
		self.seekable
	}

	pub async fn set_peeking(&mut self, peeking: bool) {
		if peeking == self.peeking.is_some() {
			return;
		}

		if peeking {
			self.peeking = Some(self.position);
			self.stream.move_data_to_beginning();
		} else {
			#[allow(clippy::unwrap_used, clippy::arithmetic_side_effects)]
			let rel = self.position - self.peeking.take().unwrap();

			/* seek should be within our buffer, no errors should occur */
			#[allow(
				clippy::unwrap_used,
				clippy::arithmetic_side_effects,
				clippy::cast_possible_wrap
			)]
			self.seek_relative(-(rel as i64)).await.unwrap();
		}
	}

	pub async fn len(&mut self) -> Result<u64> {
		self.stream.stream_len().await
	}

	pub async fn eof(&mut self) -> Result<bool> {
		if !self.stream.buffer().is_empty() {
			Ok(false)
		} else {
			Ok(self.stream.fill().await? != 0)
		}
	}
}
