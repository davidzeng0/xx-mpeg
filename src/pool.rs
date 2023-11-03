use std::{
	collections::VecDeque,
	marker::PhantomData,
	ops::{Deref, DerefMut},
	sync::{
		atomic::{AtomicU32, Ordering},
		Mutex
	}
};

use xx_core::{error::*, opt::hint::unlikely, pointer::*};

use super::*;

pub const POOL_DEFAULT_BUFFER_SIZE_AUDIO: usize = 16 * 1024;

struct Buffer {
	buffer_ref: BufferRef,
	used: usize
}

impl Buffer {
	fn from_buffer(buffer: MutPtr<buffer::Buffer>) -> Self {
		Self {
			buffer_ref: unsafe { BufferRef::from_buffer(buffer) },
			used: 0
		}
	}

	fn new(size: usize, free_fn: BufferFreeFn, user: *const ()) -> Result<Self> {
		Ok(Self {
			buffer_ref: BufferRef::with_size(size, free_fn, user)?,
			used: 0
		})
	}

	fn remaining(&self) -> usize {
		self.buffer_ref.data().len() - self.used
	}

	unsafe fn get_slice_unchecked(&mut self, size: usize) -> (BufferRef, *mut u8) {
		let ptr = self
			.buffer_ref
			.data_mut()
			.as_mut_ptr()
			.wrapping_add(self.used);
		self.used += size;
		(self.buffer_ref.clone(), ptr)
	}
}

struct PoolInner {
	buffer_size: usize,

	lock: Mutex<()>,
	current_buffer: Option<Buffer>,
	buffers: VecDeque<MutPtr<buffer::Buffer>>,

	refs: AtomicU32
}

impl PoolInner {
	pub fn new(buffer_size: usize) -> MutPtr<Self> {
		let this = Box::new(Self {
			buffer_size,

			lock: Mutex::new(()),
			current_buffer: None,
			buffers: VecDeque::new(),

			refs: AtomicU32::new(1)
		});

		MutPtr::from(Box::into_raw(this))
	}

	extern "C" fn buffer_free(user: *const (), buffer: *mut buffer::Buffer) {
		let mut pool: MutPtr<Self> = ConstPtr::from(user).cast();
		let pool = pool.as_mut();
		let lock = pool.lock.lock().unwrap();

		pool.buffers.push_back(MutPtr::from(buffer));

		/* drop lock here because we don't want to be releasing it after dec_ref
		 * calls drop */
		drop(lock);

		pool.dec_ref();
	}

	pub fn dec_ref(&mut self) {
		let prev = self.refs.fetch_sub(1, Ordering::Relaxed);

		if prev > 1 {
			return;
		}

		drop(unsafe { Box::from_raw(self) });
	}

	fn inc_ref(&mut self) {
		self.refs.fetch_add(1, Ordering::Relaxed);
	}

	#[inline(never)]
	pub fn alloc(&mut self, size: usize) -> Result<(BufferRef, *mut u8)> {
		let min_buffer_size = self.buffer_size;

		loop {
			if unlikely(size > min_buffer_size) {
				break;
			}

			let lock = self.lock.lock().unwrap();
			let lock = if self
				.current_buffer
				.as_ref()
				.is_some_and(|buf| buf.remaining() >= size)
			{
				lock
			} else {
				/* not enough spare space */
				let lock = if let Some(buffer) = self.current_buffer.take() {
					/* drop current buffer. requires lock isn't held, incase it gets
					 * requeued immediately */
					drop(lock);

					self.inc_ref();

					drop(buffer);

					self.lock.lock().unwrap()
				} else {
					lock
				};

				let buffer = match self.buffers.pop_front() {
					Some(buffer) => buffer,
					/* new allocation */
					None => break
				};

				self.current_buffer = Some(Buffer::from_buffer(buffer));

				lock
			};

			let cur = self.current_buffer.as_mut().unwrap();

			/* buffer is guaranteed to have space here */
			let buf = unsafe { cur.get_slice_unchecked(size) };

			drop(lock);

			return Ok(buf);
		}

		/* failed to get a suitably sized buffer from the pool, allocate a new one */
		let mut this = MutPtr::from(self);
		let buffer_size = size.max(min_buffer_size);

		let mut buffer = Buffer::new(
			buffer_size,
			/* for packets that are larger than our pool size (rare), just free them when
			 * dropped */
			if buffer_size == min_buffer_size {
				Self::buffer_free
			} else {
				buffer::Buffer::default_free
			},
			this.as_raw_ptr()
		)?;

		let buf = unsafe { buffer.get_slice_unchecked(size) };

		if buffer_size == this.buffer_size {
			let this = this.as_mut();
			let lock = this.lock.lock().unwrap();

			/* try insert into current_buffer, if none */
			if this.current_buffer.is_none() {
				this.current_buffer = Some(buffer);
			} else {
				drop(lock);

				/* this buffer gets restored to us later, so inc ref */
				this.inc_ref();
			}
		}

		Ok(buf)
	}
}

impl Drop for PoolInner {
	fn drop(&mut self) {
		/* make sure that dec_ref called by dropping current_buffer doesn't recurse
		 * back here */
		self.inc_ref();

		if let Some(buffer) = self.current_buffer.take() {
			/* let the current buffer get reappended to our deque */
			self.inc_ref();

			drop(buffer);
		}

		for buffer in &mut self.buffers {
			unsafe {
				buffer.free();
			}

			*buffer = MutPtr::null();
		}
	}
}

pub struct Pool {
	inner: MutPtr<PoolInner>
}

pub struct PoolHandle<'a> {
	pool: MutPtr<Pool>,
	phantom: PhantomData<&'a ()>
}

impl Deref for PoolHandle<'_> {
	type Target = Pool;

	fn deref(&self) -> &Pool {
		self.pool.as_ref()
	}
}

impl DerefMut for PoolHandle<'_> {
	fn deref_mut(&mut self) -> &mut Pool {
		self.pool.clone().as_mut()
	}
}

impl Pool {
	pub fn new(buffer_size: usize) -> Self {
		Self { inner: PoolInner::new(buffer_size) }
	}

	pub fn handle(&self) -> PoolHandle<'_> {
		PoolHandle {
			pool: ConstPtr::from(self).cast(),
			phantom: PhantomData
		}
	}

	pub fn alloc(&mut self, size: usize) -> Result<(BufferRef, *mut u8)> {
		self.inner.alloc(size)
	}
}

impl Drop for Pool {
	fn drop(&mut self) {
		self.inner.dec_ref();
	}
}
