#![allow(clippy::module_name_repetitions)]

use xx_core::static_assertions::const_assert;

use super::*;

#[allow(dead_code)]
pub struct ProbeResult {
	pub name: String,
	pub long_name: String,
	pub mime_type: String,
	pub score: f32
}

pub struct FormatContext(MutPtr<AVFormatContext>, IOContext);

ptr_deref!(FormatContext, AVFormatContext);
drop!(FormatContext, avformat_close_input);

impl FormatContext {
	pub fn new() -> Self {
		let mut this = Self(
			/* Safety: FFI call */
			alloc_with(|| unsafe { avformat_alloc_context() }),
			IOContext::new()
		);

		this.pb = this.1.as_mut_ptr();
		this
	}
}

#[asynchronous]
impl FormatContext {
	pub async fn probe(reader: &mut Reader) -> Result<Option<ProbeResult>> {
		const_assert!(DEFAULT_BUFFER_SIZE <= u32::MAX as usize);

		let read = |io: &mut IOContext| async move {
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
			/* Safety: FFI call */
			let score = result_from_av(unsafe {
				av_probe_input_buffer2(
					io.0.as_mut_ptr(),
					&mut format,
					Ptr::null().as_ptr(),
					MutPtr::null().as_mut_ptr(),
					0,
					DEFAULT_BUFFER_SIZE as u32
				)
			})?;

			#[allow(clippy::multiple_unsafe_ops_per_block)]
			/* Safety: ptr is non-null */
			let result = unsafe {
				let format = Ptr::from(format);

				#[allow(clippy::cast_precision_loss)]
				ProbeResult {
					name: cstr_to_str(ptr!(format=>name)),
					long_name: cstr_to_str(ptr!(format=>long_name)),
					mime_type: cstr_to_str(ptr!(format=>mime_type)),
					score: score as f32 / AVPROBE_SCORE_MAX as f32
				}
			};

			Ok(result)
		};

		let mut context = IOContext::new();
		let mut adapter = Adapter::new(&mut context, reader);

		match adapter.with(read).await {
			Ok(probe) => Ok(Some(probe)),
			Err(err) if err == AVError::InvalidData => Ok(None),
			Err(err) => Err(err)
		}
	}

	pub async fn open(&mut self, reader: &mut Reader) -> Result<()> {
		let mut ptr = self.0.as_mut_ptr();
		let read = |_: &mut IOContext| async move {
			/* Safety: FFI call */
			result_from_av(unsafe {
				avformat_open_input(
					&mut ptr,
					Ptr::null().as_ptr(),
					Ptr::null().as_ptr(),
					MutPtr::null().as_mut_ptr()
				)
			})?;

			/* Safety: FFI call */
			result_from_av(unsafe { avformat_find_stream_info(ptr, MutPtr::null().as_mut_ptr()) })?;

			Ok(())
		};

		let mut adapter = Adapter::new(&mut self.1, reader);

		adapter.with(read).await
	}

	pub async fn read_frame(&mut self, packet: &mut AVPacket, reader: &mut Reader) -> Result<bool> {
		let ptr = self.0.as_mut_ptr();
		let read = |_: &mut IOContext| async move {
			/* Safety: FFI call */
			result_from_av(unsafe { av_read_frame(ptr, packet.0.as_mut_ptr()) })?;

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

		let ptr = self.0.as_mut_ptr();
		let read = |_: &mut IOContext| async move {
			/* Safety: FFI call */
			result_from_av(unsafe {
				avformat_seek_file(ptr, track_index, 0, time, time, seek_flags)
			})?;

			Ok(())
		};

		let mut adapter = Adapter::new(&mut self.1, reader);

		adapter.with(read).await
	}
}
