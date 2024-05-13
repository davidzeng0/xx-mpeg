#![allow(clippy::module_name_repetitions)]

use std::mem::size_of_val;

use super::*;

pub struct Filters;

impl Filters {
	#[allow(dead_code)]
	pub fn find_by_name(name: &str) -> Option<Ptr<AVFilter>> {
		Self::find_by_name_c(&into_cstr(name))
	}

	pub fn find_by_name_c(name: &CStr) -> Option<Ptr<AVFilter>> {
		/* Safety: FFI call */
		let ptr = Ptr::from(unsafe { avfilter_get_by_name(name.as_ptr()) });

		if !ptr.is_null() {
			Some(ptr)
		} else {
			None
		}
	}
}

#[allow(missing_copy_implementations)]
pub struct FilterContext(MutPtr<AVFilterContext>);

ptr_deref!(FilterContext, AVFilterContext);

impl FilterContext {
	#[allow(dead_code)]
	pub unsafe fn set_int(&mut self, option: &str, value: i64) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { self.set_int_c(&into_cstr(option), value) }
	}

	pub unsafe fn set_int_c(&mut self, option: &CStr, value: i64) -> Result<()> {
		/* Safety: guaranteed by caller */
		result_from_av(unsafe {
			av_opt_set_int(
				self.0.as_mut_ptr().cast(),
				option.as_ptr(),
				value,
				AV_OPT_SEARCH_CHILDREN
			)
		})?;

		Ok(())
	}

	#[allow(dead_code)]
	pub unsafe fn set_binary<T>(&mut self, option: &str, value: &[T]) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { self.set_binary_c(&into_cstr(option), value) }
	}

	pub unsafe fn set_binary_c<T>(&mut self, option: &CStr, value: &[T]) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		/* Safety: guaranteed by caller */
		result_from_av(unsafe {
			av_opt_set_bin(
				self.0.as_mut_ptr().cast(),
				option.as_ptr(),
				value.as_ptr().cast(),
				size_of_val(value).try_into().unwrap(),
				AV_OPT_SEARCH_CHILDREN
			)
		})?;

		Ok(())
	}

	#[allow(dead_code)]
	pub unsafe fn set_rational(&mut self, option: &str, value: AVRational) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { self.set_rational_c(&into_cstr(option), value) }
	}

	pub unsafe fn set_rational_c(&mut self, option: &CStr, value: AVRational) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		/* Safety: guaranteed by caller */
		result_from_av(unsafe {
			av_opt_set_q(
				self.0.as_mut_ptr().cast(),
				option.as_ptr(),
				value,
				AV_OPT_SEARCH_CHILDREN
			)
		})?;

		Ok(())
	}

	#[allow(dead_code)]
	pub unsafe fn set_sample_fmt(&mut self, option: &str, value: AVSampleFormat) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { self.set_sample_fmt_c(&into_cstr(option), value) }
	}

	pub unsafe fn set_sample_fmt_c(&mut self, option: &CStr, value: AVSampleFormat) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		/* Safety: guaranteed by caller */
		result_from_av(unsafe {
			av_opt_set_sample_fmt(
				self.0.as_mut_ptr().cast(),
				option.as_ptr(),
				value,
				AV_OPT_SEARCH_CHILDREN
			)
		})?;

		Ok(())
	}

	#[allow(dead_code)]
	pub unsafe fn set_string(&mut self, option: &str, value: &str) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { self.set_string_c(&into_cstr(option), &into_cstr(value)) }
	}

	pub unsafe fn set_string_c(&mut self, option: &CStr, value: &CStr) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		/* Safety: guaranteed by caller */
		result_from_av(unsafe {
			av_opt_set(
				self.0.as_mut_ptr().cast(),
				option.as_ptr(),
				value.as_ptr(),
				AV_OPT_SEARCH_CHILDREN
			)
		})?;

		Ok(())
	}

	#[allow(dead_code)]
	pub unsafe fn set_double(&mut self, option: &str, value: f64) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { self.set_double_c(&into_cstr(option), value) }
	}

	pub unsafe fn set_double_c(&mut self, option: &CStr, value: f64) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		/* Safety: guaranteed by caller */
		result_from_av(unsafe {
			av_opt_set_double(
				self.0.as_mut_ptr().cast(),
				option.as_ptr(),
				value,
				AV_OPT_SEARCH_CHILDREN
			)
		})?;

		Ok(())
	}

	pub fn init(&mut self) -> Result<()> {
		/* Safety: FFI call */
		result_from_av(unsafe {
			avfilter_init_dict(self.0.as_mut_ptr(), MutPtr::null().as_mut_ptr())
		})?;

		Ok(())
	}

	pub fn link(&mut self, pad: u32, dst: &mut Self, dst_pad: u32) -> Result<()> {
		/* Safety: FFI call */
		result_from_av(unsafe {
			avfilter_link(self.0.as_mut_ptr(), pad, dst.0.as_mut_ptr(), dst_pad)
		})?;

		Ok(())
	}
}

pub struct Filter(pub(super) FilterContext);

av_wrapper!(
	FilterGraph,
	AVFilterGraph,
	avfilter_graph_free,
	avfilter_graph_alloc
);

impl FilterGraph {
	#[allow(dead_code)]
	pub fn create_filter(&mut self, filter: Ptr<AVFilter>, name: Option<&str>) -> FilterContext {
		let name = name.map(into_cstr);

		self.create_filter_c(filter, name.as_ref().map(AsRef::as_ref))
	}

	pub fn create_filter_c(&mut self, filter: Ptr<AVFilter>, name: Option<&CStr>) -> FilterContext {
		assert!(!filter.is_null(), "Filter is null");

		/* Safety: FFI call */
		let ptr = alloc_with(|| unsafe {
			avfilter_graph_alloc_filter(
				self.0.as_mut_ptr(),
				filter.as_ptr(),
				name.map_or(Ptr::null().as_ptr(), CStr::as_ptr)
			)
		});

		FilterContext(ptr)
	}

	pub fn config(&mut self) -> Result<()> {
		/* Safety: FFI call */
		result_from_av(unsafe {
			avfilter_graph_config(self.0.as_mut_ptr(), MutPtr::null().as_mut_ptr())
		})?;

		Ok(())
	}
}

pub struct BufferSrc(FilterContext);

impl BufferSrc {
	pub fn send_frame(&mut self, frame: AVFrame) -> Result<()> {
		/* Safety: FFI call */
		result_from_av(unsafe {
			av_buffersrc_add_frame(self.0 .0.as_mut_ptr(), frame.0.as_mut_ptr())
		})?;

		drop(frame);

		Ok(())
	}

	pub fn drain(&mut self) -> Result<()> {
		/* Safety: FFI call */
		result_from_av(unsafe {
			av_buffersrc_add_frame(self.0 .0.as_mut_ptr(), MutPtr::null().as_mut_ptr())
		})?;

		Ok(())
	}
}

deref_inner!(BufferSrc, FilterContext);

pub struct BufferSink(FilterContext);

impl BufferSink {
	pub fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		/* Safety: FFI call */
		result_from_av_maybe_none(unsafe {
			av_buffersink_get_frame(self.0 .0.as_mut_ptr(), frame.0.as_mut_ptr())
		})
	}

	pub fn set_frame_size(&mut self, frame_size: u32) {
		/* Safety: FFI call */
		unsafe { av_buffersink_set_frame_size(self.0 .0.as_mut_ptr(), frame_size) };
	}
}

deref_inner!(BufferSink, FilterContext);

pub struct AudioBufferSrc(BufferSrc);

impl AudioBufferSrc {
	pub fn new(graph: &mut FilterGraph) -> Self {
		#[allow(clippy::expect_used)]
		let filter = Filters::find_by_name_c(c"abuffer").expect("Filter not found");
		let ctx = graph.create_filter_c(filter, Some(c"in"));

		Self(BufferSrc(ctx))
	}
}

deref_inner!(AudioBufferSrc, BufferSrc);

pub struct AudioBufferSink(BufferSink);

impl AudioBufferSink {
	pub fn new(graph: &mut FilterGraph) -> Self {
		#[allow(clippy::expect_used)]
		let filter = Filters::find_by_name_c(c"abuffersink").expect("Filter not found");
		let ctx = graph.create_filter_c(filter, Some(c"out"));

		Self(BufferSink(ctx))
	}
}

deref_inner!(AudioBufferSink, BufferSink);

#[derive(Clone, Copy)]
pub struct AudioSrcOptions {
	pub time_base: Option<Rational>,
	pub sample_fmt: Option<SampleFormat>,
	pub channel_count: Option<u16>,
	pub sample_rate: Option<u32>
}

#[derive(Clone, Copy)]
pub struct AudioSinkOptions {
	pub all_channel_counts: Option<bool>,
	pub sample_fmt: Option<SampleFormat>,
	pub sample_rate: Option<u32>
}

pub struct AudioFilterGraph(FilterGraph, AudioBufferSrc, AudioBufferSink);

impl AudioFilterGraph {
	pub fn new(input: &AudioSrcOptions, output: &AudioSinkOptions) -> Self {
		let mut graph = FilterGraph::new();
		let mut src = AudioBufferSrc::new(&mut graph);
		let mut sink = AudioBufferSink::new(&mut graph);

		#[allow(clippy::multiple_unsafe_ops_per_block, clippy::unwrap_used)]
		/* Safety: initialize filters */
		unsafe {
			if let Some(time_base) = &input.time_base {
				let rational = (*time_base).into();

				src.set_rational_c(c"time_base", rational).unwrap();
			}

			if let Some(sample_fmt) = &input.sample_fmt {
				src.set_sample_fmt_c(c"sample_fmt", (*sample_fmt).into())
					.unwrap();
			}

			if let Some(channel_count) = &input.channel_count {
				src.set_int_c(c"channels", *channel_count as i64).unwrap();
			}

			if let Some(sample_rate) = &input.sample_rate {
				src.set_int_c(c"sample_rate", *sample_rate as i64).unwrap();
			}

			if let Some(all_channel_counts) = &output.all_channel_counts {
				sink.set_int_c(c"all_channel_counts", *all_channel_counts as i64)
					.unwrap();
			}

			if let Some(sample_fmt) = &output.sample_fmt {
				sink.set_binary_c(c"sample_fmts", &[*sample_fmt as u32])
					.unwrap();
			}

			if let Some(sample_rate) = &output.sample_rate {
				sink.set_binary_c(c"sample_rates", &[*sample_rate]).unwrap();
			}
		}

		Self(graph, src, sink)
	}

	pub fn set_filters(&mut self, filters: &mut [Filter]) -> Result<()> {
		self.1.init()?;
		self.2.init()?;

		let mut prev = &mut self.1 .0 .0;

		for filter in filters {
			filter.0.init()?;
			prev.link(0, &mut filter.0, 0)?;
			prev = &mut filter.0;
		}

		prev.link(0, &mut self.2 .0 .0, 0)?;

		self.0.config()
	}

	pub fn send_frame(&mut self, frame: AVFrame) -> Result<()> {
		self.1.send_frame(frame)
	}

	pub fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		self.2.receive_frame(frame)
	}

	pub fn set_frame_size(&mut self, frame_size: u32) {
		self.2.set_frame_size(frame_size);
	}

	pub fn drain(&mut self) -> Result<()> {
		self.1.drain()
	}
}

deref_inner!(AudioFilterGraph, FilterGraph);
