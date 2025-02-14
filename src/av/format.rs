use xx_core::macros::const_assert;

use super::*;

#[allow(dead_code)]
pub struct ProbeResult {
	pub name: String,
	pub long_name: String,
	pub mime_type: String,
	pub score: f32,
	pub format: NonNull<AVInputFormat>
}

struct Guard(MutPtr<AVFormatContext>);

ptr_deref!(Guard, AVFormatContext);
drop!(Guard, avformat_close_input);

pub struct FormatContext(MutPtr<AVFormatContext>, IoContext);

ptr_deref!(FormatContext, AVFormatContext);
drop!(FormatContext, avformat_close_input);

impl FormatContext {
	pub fn new() -> Self {
		let context = IoContext::new();

		let mut this = Self(alloc_with(|| ffi!(avformat_alloc_context)).into(), context);

		this.pb = this.1.as_mut_ptr();
		this.flags |= AVFMT_FLAG_CUSTOM_IO;
		this
	}
}

#[asynchronous]
impl FormatContext {
	pub async fn probe(reader: &mut Reader) -> Result<Option<ProbeResult>> {
		const_assert!(DEFAULT_BUFFER_SIZE <= u32::MAX as usize);

		let read = |io: &mut IoContext| async move {
			/// # Safety
			/// valid cstr pointer
			///
			/// # Panics
			/// if the resulting bytes is not utf8
			unsafe fn cstr_to_str(cstr: *const c_char) -> String {
				if cstr.is_null() {
					return String::new();
				}

				/* Safety: guaranteed by caller */
				let str = unsafe { CStr::from_ptr(cstr) };

				#[allow(clippy::unwrap_used)]
				str.to_str().unwrap().to_string()
			}

			let mut format = Ptr::null().as_ptr();

			#[allow(clippy::cast_possible_truncation)]
			let score = ffi!(
				av_probe_input_buffer2,
				io.as_mut_ptr(),
				&mut format,
				Ptr::null().as_ptr(),
				MutPtr::null().as_mut_ptr(),
				0,
				DEFAULT_BUFFER_SIZE as u32
			)?;

			#[allow(clippy::multiple_unsafe_ops_per_block)]
			/* Safety: ptr is non-null */
			let result = unsafe {
				let format = NonNull::new_unchecked(format.into());

				#[allow(clippy::cast_precision_loss)]
				ProbeResult {
					name: cstr_to_str(ptr!(format=>name)),
					long_name: cstr_to_str(ptr!(format=>long_name)),
					mime_type: cstr_to_str(ptr!(format=>mime_type)),
					score: score as f32 / AVPROBE_SCORE_MAX as f32,
					format
				}
			};

			Ok(result)
		};

		let mut context = IoContext::new();
		let mut adapter = Adapter::new(&mut context, reader);

		match adapter.with(read).await {
			Ok(probe) => Ok(Some(probe)),
			Err(err) if err == AVError::InvalidData => Ok(None),
			Err(err) => Err(err)
		}
	}

	pub async fn open(&mut self, reader: &mut Reader) -> Result<()> {
		let mut ptr = self.as_mut_ptr();
		let pb = self.1.as_mut_ptr();
		let guard = Guard(alloc_with(|| ffi!(avformat_alloc_context)).into());

		let read = |_: &mut IoContext| async {
			let result = ffi!(
				avformat_open_input,
				&mut ptr,
				Ptr::null().as_ptr(),
				Ptr::null().as_ptr(),
				MutPtr::null().as_mut_ptr()
			);

			if let Err(err) = result {
				self.0 = guard.0;

				forget(guard);

				/* Safety: valid ptr
				 * set pb to prevent null pointer use if other functions are called
				 */
				#[allow(clippy::multiple_unsafe_ops_per_block)]
				unsafe {
					ptr!(self.0=>pb = pb);
					ptr!(self.0=>flags |= AVFMT_FLAG_CUSTOM_IO);
				}

				return Err(err);
			}

			ffi!(avformat_find_stream_info, ptr, MutPtr::null().as_mut_ptr())?;

			Ok(())
		};

		let mut adapter = Adapter::new(&mut self.1, reader);

		adapter.with(read).await
	}

	pub async fn read_frame(&mut self, packet: &mut AVPacket, reader: &mut Reader) -> Result<bool> {
		let ptr = self.as_mut_ptr();
		let read = |_: &mut IoContext| async {
			ffi!(av_read_frame, ptr, packet.as_mut_ptr())?;

			Ok(())
		};

		let mut adapter = Adapter::new(&mut self.1, reader);

		match adapter.with(read).await {
			Ok(()) => Ok(true),
			Err(err) if err == AVError::EndOfFile => Ok(false),
			Err(err) => Err(err)
		}
	}

	pub async fn seek(
		&mut self, track_index: u32, timecode: u64, flags: BitFlags<SeekFlag>, reader: &mut Reader
	) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		let track_index = track_index.try_into().unwrap();

		#[allow(clippy::unwrap_used)]
		let time = timecode.try_into().unwrap();

		let mut seek_flags = 0;

		if flags.intersects(SeekFlag::Any) {
			seek_flags |= AVSEEK_FLAG_ANY;
		}

		let ptr = self.as_mut_ptr();
		let read = |_: &mut IoContext| async move {
			ffi!(
				avformat_seek_file,
				ptr,
				track_index,
				0,
				time,
				time,
				seek_flags
			)?;

			Ok(())
		};

		let mut adapter = Adapter::new(&mut self.1, reader);

		adapter.with(read).await
	}
}
