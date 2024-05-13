use std::io::{Cursor, Write};

use super::*;

macro_rules! new_filter {
	($name:literal) => {
		pub fn new(graph: &mut FilterGraph) -> Self {
			#[allow(clippy::expect_used)]
			let filter = Filters::find_by_name_c($name).expect("Filter not found");
			let ctx = graph.create_filter_c(filter, None);

			Self(Filter(ctx))
		}

		pub const fn into_inner(self) -> Filter {
			self.0
		}
	};
}

pub struct Volume(Filter);

impl Volume {
	new_filter!(c"volume");

	pub fn set_volume(&mut self, volume: f64) -> Result<()> {
		let mut buf = Cursor::new([0u8; 32]);
		let _ = buf.write_fmt(format_args!("{}", volume));

		let Ok(1) = buf.write(&[0]) else {
			return Err(fmt_error!("Failed to write string"));
		};

		#[allow(clippy::cast_possible_truncation, clippy::unwrap_used)]
		let volume = CStr::from_bytes_until_nul(&buf.get_ref()[0..buf.position() as usize]).unwrap();

		/* Safety: set volume */
		unsafe { self.0 .0.set_string_c(c"volume", volume) }
	}
}

pub struct SetRate(Filter);

impl SetRate {
	new_filter!(c"asetrate");

	pub fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
		/* Safety: set rate */
		unsafe { self.0 .0.set_int_c(c"sample_rate", rate as i64) }
	}
}

pub struct Tempo(Filter);

impl Tempo {
	new_filter!(c"atempo");

	pub fn set_tempo(&mut self, tempo: f64) -> Result<()> {
		/* Safety: set tempo */
		unsafe { self.0 .0.set_double_c(c"tempo", tempo) }
	}
}

pub struct FirEqualizer(Filter);

fn join_gain_entries<F, Output>(entries: &[(f64, f64)], func: F) -> Result<Output>
where
	F: FnOnce(&CStr) -> Result<Output>
{
	let mut buf = Cursor::new([0u8; 1024]);

	for (i, (frequency, gain)) in entries.iter().enumerate() {
		if i > 0 {
			let _ = buf.write(b"; ");
		}

		let _ = buf.write_fmt(format_args!("entry({}, {})", frequency, gain));
	}

	let Ok(1) = buf.write(&[0]) else {
		return Err(fmt_error!("Failed to write string"));
	};

	#[allow(clippy::cast_possible_truncation, clippy::unwrap_used)]
	let joined = CStr::from_bytes_until_nul(&buf.get_ref()[0..buf.position() as usize]).unwrap();

	func(joined)
}

impl FirEqualizer {
	new_filter!(c"firequalizer");

	pub fn set_gain_entries(&mut self, entries: &[(f64, f64)]) -> Result<()> {
		/* Safety: set gain entries */
		join_gain_entries(entries, |str| unsafe {
			self.0 .0.set_string_c(c"gain_entry", str)
		})
	}

	pub fn set_delay(&mut self, delay: f64) -> Result<()> {
		/* Safety: set delay */
		unsafe { self.0 .0.set_double_c(c"delay", delay) }
	}

	pub fn set_accuracy(&mut self, accuracy: f64) -> Result<()> {
		/* Safety: set accuracy */
		unsafe { self.0 .0.set_double_c(c"accuracy", accuracy) }
	}

	pub fn set_multi_channel(&mut self, multi: bool) -> Result<()> {
		/* Safety: set multi */
		unsafe { self.0 .0.set_int_c(c"multi", multi as i64) }
	}

	pub fn set_zero_phase(&mut self, zero_phase: bool) -> Result<()> {
		/* Safety: set zero phase */
		unsafe { self.0 .0.set_int_c(c"zero_phase", zero_phase as i64) }
	}

	pub fn set_use_fft_two_channels(&mut self, enabled: bool) -> Result<()> {
		/* Safety: set zero phase */
		unsafe { self.0 .0.set_int_c(c"fft2", enabled as i64) }
	}
}

pub struct Resample(Filter);

impl Resample {
	new_filter!(c"aresample");

	pub fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
		/* Safety: set rate */
		unsafe { self.0 .0.set_int_c(c"sample_rate", rate as i64) }
	}
}

pub struct Pulsator(Filter);

impl Pulsator {
	new_filter!(c"apulsator");

	pub fn set_level_in(&mut self, gain: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"level_in", gain) }
	}

	pub fn set_level_out(&mut self, gain: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"level_out", gain) }
	}

	pub fn set_amount(&mut self, modulation: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"amount", modulation) }
	}

	pub fn set_offset_left(&mut self, offset: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"offset_l", offset) }
	}

	pub fn set_offset_right(&mut self, offset: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"offset_r", offset) }
	}

	pub fn set_width(&mut self, width: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"width", width) }
	}

	pub fn set_frequency(&mut self, frequency: f64) -> Result<()> {
		/* Safety: set option */
		unsafe { self.0 .0.set_double_c(c"hz", frequency) }
	}
}

pub struct Echo(Filter);

fn join_floats<F, Output>(floats: &[f32], func: F) -> Result<Output>
where
	F: FnOnce(&CStr) -> Result<Output>
{
	let mut buf = Cursor::new([0u8; 1024]);

	for (i, value) in floats.iter().enumerate() {
		if i > 0 {
			let _ = buf.write(b"|");
		}

		let _ = buf.write_fmt(format_args!("{}", value));
	}

	let Ok(1) = buf.write(&[0]) else {
		return Err(fmt_error!("Failed to write string"));
	};

	#[allow(clippy::cast_possible_truncation, clippy::unwrap_used)]
	let joined = CStr::from_bytes_until_nul(&buf.get_ref()[0..buf.position() as usize]).unwrap();

	func(joined)
}

impl Echo {
	new_filter!(c"aecho");

	pub fn set_input_gain(&mut self, gain: f32) -> Result<()> {
		/* Safety: set gain */
		unsafe { self.0 .0.set_double_c(c"in_gain", gain as f64) }
	}

	pub fn set_output_gain(&mut self, gain: f32) -> Result<()> {
		/* Safety: set gain */
		unsafe { self.0 .0.set_double_c(c"out_gain", gain as f64) }
	}

	pub fn set_delays(&mut self, delays: &[f32]) -> Result<()> {
		/* Safety: set delays */
		join_floats(delays, |str| unsafe {
			self.0 .0.set_string_c(c"delays", str)
		})
	}

	pub fn set_decays(&mut self, decays: &[f32]) -> Result<()> {
		/* Safety: set decays */
		join_floats(decays, |str| unsafe {
			self.0 .0.set_string_c(c"decays", str)
		})
	}
}
