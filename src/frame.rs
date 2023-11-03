use std::{
	alloc::{dealloc, Layout},
	mem::{align_of, size_of},
	ptr::{drop_in_place, null_mut}
};

use enumflags2::BitFlags;
use ffmpeg_sys_next::AV_NUM_DATA_POINTERS;

use super::*;

const DATA_POINTERS: usize = AV_NUM_DATA_POINTERS as usize;

pub struct Frame {
	line_size: [i32; DATA_POINTERS],
	data: [*mut u8; DATA_POINTERS],
	bufs: [*mut BufferRef; DATA_POINTERS],

	extended_buf: *mut *mut BufferRef,
	extended_buf_size: usize,
	extended_data: *mut *mut u8,

	pub time_base: Rational,
	pub duration: u64,
	pub timestamp: i64,
	pub samples: u32,
	pub channels: u16,
	pub sample_rate: u32,
	pub channel_layout: u64,
	pub flags: BitFlags<FrameFlag>,
	pub format: i32
}

impl Frame {
	pub fn new() -> Self {
		Self {
			line_size: [0i32; DATA_POINTERS],
			data: [null_mut(); DATA_POINTERS],
			bufs: [null_mut(); DATA_POINTERS],

			extended_buf: null_mut(),
			extended_buf_size: 0,
			extended_data: null_mut(),

			time_base: Rational::default(),
			duration: 0,
			timestamp: UNKNOWN_TIMESTAMP,
			samples: 0,
			channels: 0,
			sample_rate: 0,
			channel_layout: 0,
			flags: BitFlags::default(),
			format: -1
		}
	}

	pub fn line_size(&self) -> &[i32; DATA_POINTERS] {
		&self.line_size
	}

	pub fn line_size_mut(&mut self) -> &mut [i32; DATA_POINTERS] {
		&mut self.line_size
	}

	pub fn data(&self) -> &[*mut u8; DATA_POINTERS] {
		&self.data
	}

	pub fn data_mut(&mut self) -> &mut [*mut u8; DATA_POINTERS] {
		&mut self.data
	}

	pub fn bufs(&self) -> &[*mut BufferRef; DATA_POINTERS] {
		&self.bufs
	}

	pub fn bufs_mut(&mut self) -> &mut [*mut BufferRef; DATA_POINTERS] {
		&mut self.bufs
	}

	pub fn extended_bufs(&self) -> *mut *mut BufferRef {
		self.extended_buf
	}

	pub fn extended_bufs_size(&self) -> usize {
		self.extended_buf_size
	}

	pub fn extended_data(&self) -> *mut *mut u8 {
		self.extended_data
	}

	pub unsafe fn set_extended_bufs(&mut self, bufs: *mut *mut BufferRef) {
		self.extended_buf = bufs;
	}

	pub unsafe fn set_extended_bufs_size(&mut self, size: usize) {
		self.extended_buf_size = size;
	}

	pub unsafe fn set_extended_data(&mut self, data: *mut *mut u8) {
		self.extended_data = data;
	}
}

impl Drop for Frame {
	fn drop(&mut self) {
		unsafe fn buf_ptr_unref(buf: *mut BufferRef) {
			drop_in_place(buf);
			dealloc(buf.cast(), Layout::new::<*mut BufferRef>());
		}

		unsafe {
			for buf in &mut self.bufs {
				if buf.is_null() {
					break;
				}

				buf_ptr_unref(*buf);
			}

			if !self.extended_data.is_null() {
				let layout = Layout::from_size_align_unchecked(
					size_of::<*mut u8>() * (self.extended_buf_size + DATA_POINTERS),
					align_of::<*mut u8>()
				);

				dealloc(self.extended_data.cast(), layout);

				self.extended_data = null_mut();
			}

			if !self.extended_buf.is_null() {
				for i in 0..self.extended_buf_size {
					buf_ptr_unref(*self.extended_buf.wrapping_add(i));
				}

				let layout = Layout::from_size_align_unchecked(
					self.extended_buf_size * size_of::<*mut BufferRef>(),
					align_of::<*mut BufferRef>()
				);

				dealloc(self.extended_buf.cast(), layout);
			}
		}
	}
}
