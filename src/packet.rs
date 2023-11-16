use std::{ptr::null_mut, slice};

use enumflags2::BitFlags;
use xx_core::{error::*, pointer::Ptr};
use xx_pulse::*;

use super::*;

pub struct Packet {
	buffer: BufferRef,
	data: *mut u8,
	size: usize,

	pub time_base: Rational,
	pub duration: u64,
	pub timestamp: i64,
	pub track_index: u32,
	pub flags: BitFlags<PacketFlag>,
	pub zero_padding: usize
}

#[async_fn]
impl Packet {
	pub fn new() -> Self {
		Self {
			buffer: BufferRef::new(),
			data: null_mut(),
			size: 0,

			time_base: Rational::default(),
			duration: 0,
			timestamp: UNKNOWN_TIMESTAMP,
			track_index: 0,
			flags: BitFlags::default(),
			zero_padding: 0
		}
	}

	pub unsafe fn get_buffer(&self) -> &BufferRef {
		&self.buffer
	}

	pub unsafe fn set_buffer(&mut self, buffer_ref: BufferRef, data: *mut u8, size: usize) {
		self.buffer = buffer_ref;
		self.data = data;
		self.size = size;
	}

	pub fn data(&self) -> &[u8] {
		unsafe { slice::from_raw_parts(self.data, self.size) }
	}

	pub fn data_mut(&mut self) -> &mut [u8] {
		unsafe { slice::from_raw_parts_mut(self.data, self.size) }
	}

	pub fn alloc(size: usize, padding: usize, pool: Option<&Pool>) -> Result<Self> {
		let total_size = size.checked_add(padding).unwrap();
		let mut this = Self::new();

		let (buffer, data) = if let Some(pool) = pool {
			pool.handle().alloc(total_size)?
		} else {
			let mut buffer = BufferRef::with_size(total_size, Buffer::default_free, Ptr::null())?;
			let data = buffer.data_mut().as_mut_ptr();

			(buffer, data)
		};

		this.buffer = buffer;
		this.data = data;
		this.size = size;
		this.zero_padding = padding;

		let padding = unsafe { this.data_mut().get_unchecked_mut(size..) };

		/* optimizes to memset */
		padding.iter_mut().for_each(|b| *b = 0);

		Ok(this)
	}
}
