use super::*;

pub struct Filters;

impl Filters {
	#[allow(dead_code)]
	pub fn find_by_name(name: &str) -> Option<NonNull<AVFilter>> {
		Self::find_by_name_c(&into_cstr(name))
	}

	pub fn find_by_name_c(name: &CStr) -> Option<NonNull<AVFilter>> {
		NonNull::new(ffi!(avfilter_get_by_name, name.as_ptr()).into())
	}
}

#[allow(missing_copy_implementations)]
pub struct FilterContext(MutNonNull<AVFilterContext>);

ptr_deref!(FilterContext, AVFilterContext);

impl FilterContext {
	pub fn init(&mut self) -> Result<()> {
		ffi!(
			avfilter_init_dict,
			self.as_mut_ptr(),
			MutPtr::null().as_mut_ptr()
		)?;

		Ok(())
	}

	#[allow(clippy::needless_pass_by_ref_mut)]
	pub fn options(&mut self) -> Object<'_> {
		Object::from(self.0)
	}

	pub fn link(&mut self, pad: u32, dst: &mut Self, dst_pad: u32) -> Result<()> {
		ffi!(
			avfilter_link,
			self.as_mut_ptr(),
			pad,
			dst.as_mut_ptr(),
			dst_pad
		)?;

		Ok(())
	}
}

av_wrapper!(
	FilterGraph,
	AVFilterGraph,
	avfilter_graph_free,
	avfilter_graph_alloc
);

impl FilterGraph {
	#[allow(dead_code)]
	pub fn create_filter(
		&mut self, filter: NonNull<AVFilter>, name: Option<&str>
	) -> FilterContext {
		let name = name.map(into_cstr);

		self.create_filter_c(filter, name.as_ref().map(AsRef::as_ref))
	}

	pub fn create_filter_c(
		&mut self, filter: NonNull<AVFilter>, name: Option<&CStr>
	) -> FilterContext {
		let ptr = alloc_with(|| {
			ffi!(
				avfilter_graph_alloc_filter,
				self.as_mut_ptr(),
				filter.as_ptr(),
				name.map_or(Ptr::null().as_ptr(), CStr::as_ptr)
			)
		});

		FilterContext(ptr)
	}

	pub fn config(&mut self) -> Result<()> {
		ffi!(
			avfilter_graph_config,
			self.as_mut_ptr(),
			MutPtr::null().as_mut_ptr()
		)?;

		Ok(())
	}
}

#[derive(Clone)]
pub struct AudioSrcOptions {
	pub time_base: Option<Rational>,
	pub sample_fmt: Option<SampleFormat>,
	pub ch_layout: Option<ChannelLayout>,
	pub sample_rate: Option<u32>
}

#[derive(Clone)]
pub struct AudioSinkOptions {
	pub ch_layout: Option<ChannelLayout>,
	pub sample_fmt: Option<SampleFormat>,
	pub sample_rate: Option<u32>
}

pub struct AudioFilterGraph(FilterGraph, AudioBufferSrc, AudioBufferSink);

impl AudioFilterGraph {
	/// # Panics
	/// if the sample rate cannot fit into an i32
	pub fn new(threads: u16, input: &AudioSrcOptions, output: &AudioSinkOptions) -> Self {
		let mut graph = FilterGraph::new();

		graph.nb_threads = threads as i32;

		let mut src = AudioBufferSrc::new(&mut graph);
		let mut sink = AudioBufferSink::new(&mut graph);

		if let Some(time_base) = input.time_base {
			src.set_time_base(time_base);
		}

		if let Some(sample_fmt) = input.sample_fmt {
			src.set_sample_fmt(sample_fmt);
		}

		if let Some(ch_layout) = &input.ch_layout {
			src.set_ch_layout(ch_layout);
		}

		if let Some(sample_rate) = input.sample_rate {
			#[allow(clippy::unwrap_used)]
			src.set_sample_rate(sample_rate.try_into().unwrap());
		}

		if let Some(ch_layout) = &output.ch_layout {
			sink.set_ch_layouts(&[ch_layout.clone()]);
		} else {
			sink.set_all_channel_counts(true);
		}

		if let Some(sample_fmt) = output.sample_fmt {
			sink.set_sample_fmts(&[sample_fmt]);
		}

		if let Some(sample_rate) = output.sample_rate {
			#[allow(clippy::unwrap_used)]
			sink.set_sample_rates(&[sample_rate.try_into().unwrap()]);
		}

		Self(graph, src, sink)
	}

	pub fn set_filters<'a, I>(&mut self, filters: I) -> Result<()>
	where
		I: IntoIterator<Item = &'a mut FilterContext>
	{
		self.1.init()?;
		self.2.init()?;

		let mut prev = &mut **self.1;

		for filter in filters {
			filter.init()?;
			prev.link(0, filter, 0)?;
			prev = filter;
		}

		prev.link(0, &mut self.2, 0)?;

		self.0.config()
	}

	pub fn send_frame(&mut self, frame: AVFrame) -> Result<()> {
		/* Safety: frame is valid */
		unsafe { self.1.send_frame(frame) }
	}

	pub fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		/* Safety: frame is valid */
		unsafe { self.2.receive_frame(frame) }
	}

	pub fn set_frame_size(&mut self, frame_size: u32) {
		self.2.set_frame_size(frame_size);
	}

	pub fn drain(&mut self) -> Result<()> {
		self.1.drain()
	}
}

deref_inner!(AudioFilterGraph, FilterGraph);
