use super::*;

pub mod filters;

pub struct AudioFilterGraph(av::AudioFilterGraph);

pub trait Filter {
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::Filter>;
}

#[derive(Clone, Copy)]
pub struct AudioSrcOptions {
	pub time_base: Option<Rational>,
	pub sample_fmt: SampleFormat,
	pub channel_count: u16,
	pub sample_rate: u32
}

#[derive(Clone, Copy)]
pub struct AudioSinkOptions {
	pub sample_fmt: SampleFormat,
	pub sample_rate: u32,
	pub frame_size: Option<u32>
}

impl AudioFilterGraph {
	pub fn new(
		input: &AudioSrcOptions, output: &AudioSinkOptions, filters: &[&dyn Filter]
	) -> Result<Self> {
		let av_in = av::AudioSrcOptions {
			time_base: input.time_base,
			sample_fmt: Some(input.sample_fmt),
			channel_count: Some(input.channel_count),
			sample_rate: Some(input.sample_rate)
		};

		let av_out = av::AudioSinkOptions {
			all_channel_counts: Some(false),
			sample_fmt: Some(output.sample_fmt),
			sample_rate: Some(output.sample_rate)
		};

		let mut graph = av::AudioFilterGraph::new(&av_in, &av_out);
		let mut filt = Vec::new();

		for filter in filters {
			filt.push(filter.create_filter(&mut graph)?);
		}

		graph.nb_threads = 1;
		graph.set_filters(&mut filt)?;

		if let Some(size) = output.frame_size {
			graph.set_frame_size(size);
		}

		Ok(Self(graph))
	}

	pub fn send_frame(&mut self, frame: Frame) -> Result<()> {
		self.0.send_frame(frame.data)
	}

	pub fn receive_frame(&mut self) -> Result<Option<Frame>> {
		let mut frame = Frame::new();

		Ok(match self.0.receive_frame(&mut frame.data)? {
			true => Some(frame),
			false => None
		})
	}

	pub fn set_frame_size(&mut self, frame_size: u32) {
		self.0.set_frame_size(frame_size);
	}

	pub fn drain(&mut self) -> Result<()> {
		self.0.drain()
	}
}
