use std::{
	alloc::{alloc, dealloc, Layout},
	mem::{align_of, size_of, zeroed},
	slice,
	sync::atomic::{AtomicU32, Ordering}
};

use enumflags2::{bitflags, BitFlags};
use ffmpeg_sys_next::*;
use xx_core::{error::*, os::error::ErrorCodes, pointer::MutPtr};

#[bitflags]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
enum BufferFlag {
	Reallocatable = 1 << 0,
	NoFree        = 1 << 1
}

pub type BufferFreeFn = extern "C" fn(*const (), *mut Buffer);

#[repr(C)]
pub struct Buffer {
	/* used to store data, but in our case a self-reference */
	data: *mut u8,
	size: usize,
	refs: AtomicU32,
	free_fn: BufferFreeFn,
	user: *const (),
	flags: u32,
	flags_internal: u32
}

impl Buffer {
	pub extern "C" fn default_free(_: *const (), _: *mut Buffer) {
		/* this is our buffer, not libav's, otherwise we wouldn't be here. in
		 * that case, nothing to do, as the data is tied to the buffer, and
		 * will be freed when the buffer frees itself */
	}

	fn layout_for_size(size: usize) -> Layout {
		unsafe {
			Layout::from_size_align_unchecked(size + size_of::<Buffer>(), align_of::<Buffer>())
		}
	}

	fn new(len: usize, free_fn: BufferFreeFn, user: *const ()) -> Result<MutPtr<Self>> {
		let mut buffer = MutPtr::from(unsafe { alloc(Self::layout_for_size(len)) } as *mut Buffer);

		if buffer.is_null() {
			return Err(Error::from_raw_os_error(ErrorCodes::NoMem as i32));
		}

		*buffer = Self {
			data: buffer.as_ptr_mut().cast(),
			size: len,
			/* refs gets incremented when a buffer ref is constructed */
			refs: AtomicU32::new(0),
			free_fn,
			user,
			flags: 0,
			flags_internal: BufferFlag::NoFree as u32
		};

		Ok(buffer)
	}

	fn data(&mut self) -> &mut [u8] {
		unsafe {
			slice::from_raw_parts_mut(
				self.data.wrapping_add(size_of::<Buffer>()).cast(),
				self.size
			)
		}
	}

	fn inc_ref(&mut self) {
		self.refs.fetch_add(1, Ordering::Relaxed);
	}

	fn internal_flags(&self) -> BitFlags<BufferFlag> {
		unsafe { BitFlags::from_bits_unchecked(self.flags_internal) }
	}

	fn dec_ref(&mut self) {
		let prev = self.refs.fetch_sub(1, Ordering::Relaxed);

		if prev > 1 {
			return;
		}

		let self_free = !self.internal_flags().intersects(BufferFlag::NoFree);

		(self.free_fn)(self.user, self.data.cast());

		if self_free {
			unsafe {
				self.free();
			}
		}
	}

	pub unsafe fn free(&mut self) {
		let this = MutPtr::from(self);

		if MutPtr::from(this.data).cast() == this {
			unsafe {
				dealloc(this.as_ptr_mut().cast(), Self::layout_for_size(this.size));
			}
		} else {
			unsafe {
				dealloc(this.as_ptr_mut().cast(), Layout::new::<Self>());
			}
		}
	}
}

pub struct BufferRef {
	buffer: AVBufferRef
}

impl BufferRef {
	fn buffer(&mut self) -> MutPtr<Buffer> {
		MutPtr::from(self.buffer.buffer.cast())
	}

	pub fn new() -> Self {
		Self { buffer: unsafe { zeroed() } }
	}

	pub fn with_size(size: usize, free_fn: BufferFreeFn, user: *const ()) -> Result<Self> {
		Ok(unsafe { Self::from_buffer(Buffer::new(size, free_fn, user)?) })
	}

	pub unsafe fn from_buffer(mut buffer: MutPtr<Buffer>) -> Self {
		buffer.inc_ref();

		Self {
			buffer: AVBufferRef {
				buffer: buffer.as_ptr_mut().cast(),
				data: buffer.data().as_mut_ptr(),
				size: buffer.data().len()
			}
		}
	}

	/// Makes a new BufferRef from an AVBufferRef
	pub fn from_buffer_ref(buffer: AVBufferRef) -> Self {
		let mut this = Self { buffer };

		this.buffer().inc_ref();
		this
	}

	pub fn data(&self) -> &[u8] {
		unsafe { slice::from_raw_parts(self.buffer.data, self.buffer.size) }
	}

	pub fn data_mut(&mut self) -> &mut [u8] {
		unsafe { slice::from_raw_parts_mut(self.buffer.data, self.buffer.size) }
	}

	/// Gets the buffer ref, without increasing the ref count
	pub unsafe fn get_ref(&self) -> &AVBufferRef {
		&self.buffer
	}
}

impl Default for BufferRef {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for BufferRef {
	fn clone(&self) -> Self {
		let mut this = Self { buffer: self.buffer };

		if !this.buffer().is_null() {
			this.buffer().inc_ref();
		}

		this
	}
}

impl Drop for BufferRef {
	fn drop(&mut self) {
		if !self.buffer().is_null() {
			self.buffer().dec_ref();
			self.buffer = unsafe { zeroed() };
		}
	}
}
