use super::*;

#[derive(Clone, Copy, PartialEq)]
pub struct Volume(pub f64);

impl Default for Volume {
	fn default() -> Self {
		Self(1.0)
	}
}

impl Filter for Volume {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut volume = av::Volume::new(graph);

		volume.set_volume(self.0);

		Ok(volume.into_filter())
	}
}

#[derive(Clone, Copy, PartialEq)]
pub struct SetRate(pub u32);

impl Filter for SetRate {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut rate = av::SetRate::new(graph);

		rate.set_sample_rate(self.0);

		Ok(rate.into_filter())
	}
}

#[derive(Clone, Copy, PartialEq)]
pub struct Tempo(pub f64);

impl Default for Tempo {
	fn default() -> Self {
		Self(1.0)
	}
}

impl Filter for Tempo {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut tempo = av::Tempo::new(graph);

		tempo.set_tempo(self.0);

		Ok(tempo.into_filter())
	}
}

#[derive(Clone, PartialEq)]
pub struct FirEqualizer {
	pub gain_entries: Vec<(f64, f64)>,
	pub delay: f64,
	pub accuracy: f64,
	pub multi_channel: bool,
	pub zero_phase: bool,
	pub use_fft_two_channels: bool
}

impl Default for FirEqualizer {
	fn default() -> Self {
		Self {
			gain_entries: Vec::new(),
			delay: 0.01,
			accuracy: 5.0,
			multi_channel: false,
			zero_phase: false,
			use_fft_two_channels: false
		}
	}
}

impl Filter for FirEqualizer {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut eq = av::FirEqualizer::new(graph);

		eq.set_gain_entries(&self.gain_entries);
		eq.set_delay(self.delay);
		eq.set_accuracy(self.accuracy);
		eq.set_multi_channel(self.multi_channel);
		eq.set_zero_phase(self.zero_phase);
		eq.set_use_fft_two_channels(self.use_fft_two_channels);

		Ok(eq.into_filter())
	}
}

#[derive(Clone, Copy, PartialEq)]
pub struct Resample(pub u32);

impl Filter for Resample {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut resample = av::Resample::new(graph);

		resample.set_sample_rate(self.0);

		Ok(resample.into_filter())
	}
}

#[derive(Clone, Copy, PartialEq)]
pub struct Pulsator {
	pub level_in: f64,
	pub level_out: f64,
	pub modulation: f64,
	pub offset_left: f64,
	pub offset_right: f64,
	pub width: f64,
	pub frequency: f64
}

impl Default for Pulsator {
	fn default() -> Self {
		Self {
			level_in: 1.0,
			level_out: 1.0,
			modulation: 1.0,
			offset_left: 0.0,
			offset_right: 0.5,
			width: 1.0,
			frequency: 2.0
		}
	}
}

impl Filter for Pulsator {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut pulsator = av::Pulsator::new(graph);

		pulsator.set_level_in(self.level_in);
		pulsator.set_level_out(self.level_out);
		pulsator.set_modulation(self.modulation);
		pulsator.set_offset_left(self.offset_left);
		pulsator.set_offset_right(self.offset_right);
		pulsator.set_width(self.width);
		pulsator.set_frequency(self.frequency);

		Ok(pulsator.into_filter())
	}
}

#[derive(Clone, PartialEq)]
pub struct Echo {
	pub input_gain: f32,
	pub output_gain: f32,
	pub delays: Vec<f32>,
	pub decays: Vec<f32>
}

impl Default for Echo {
	fn default() -> Self {
		Self {
			input_gain: 0.6,
			output_gain: 0.3,
			delays: vec![1000.0],
			decays: vec![0.5]
		}
	}
}

impl Filter for Echo {
	#[allow(clippy::unwrap_used)]
	fn create_filter(&self, graph: &mut av::FilterGraph) -> Result<av::FilterContext> {
		let mut echo = av::Echo::new(graph);

		echo.set_input_gain(self.input_gain);
		echo.set_output_gain(self.output_gain);
		echo.set_delays(&self.delays);
		echo.set_decays(&self.decays);

		Ok(echo.into_filter())
	}
}
