use std::fmt::{Arguments, Display};
use std::io::Write;

use xx_core::io::UninitBuf;

use super::*;

macro_rules! new_filter {
	($name:literal) => {
		pub fn new(graph: &mut FilterGraph) -> Self {
			#[allow(clippy::expect_used)]
			let filter = Filters::find_by_name_c($name).expect("Filter not found");
			let ctx = graph.create_filter_c(filter, Some($name));

			Self(ctx)
		}

		pub const fn into_filter(self) -> FilterContext {
			self.0
		}
	};
}

trait Format {
	fn write<W>(&self, writer: &mut W)
	where
		W: Write;
}

impl<T: Display + ?Sized> Format for T {
	fn write<W>(&self, writer: &mut W)
	where
		W: Write
	{
		let _ = writer.write_fmt(format_args!("{}", self));
	}
}

impl Format for ChannelLayout {
	fn write<W>(&self, writer: &mut W)
	where
		W: Write
	{
		let mut buf = [0u8; 2048];

		#[allow(clippy::unwrap_used)]
		let len = ffi!(
			av_channel_layout_describe,
			&self.into(),
			buf.as_mut_ptr().cast(),
			buf.len()
		)
		.unwrap();

		#[allow(clippy::cast_sign_loss, clippy::arithmetic_side_effects)]
		let _ = writer.write(&buf[0..(len - 1) as usize]);
	}
}

/// # Panics
/// if writing the string fails
fn format_cstr<const N: usize, W, F>(write: W, func: F)
where
	W: FnOnce(&mut UninitBuf<N>),
	F: FnOnce(&CStr)
{
	let mut buf = UninitBuf::new();

	write(&mut buf);

	assert_eq!(buf.extend_from_slice(&[0]), 1, "Failed to write string");

	#[allow(clippy::cast_possible_truncation, clippy::unwrap_used)]
	let str = CStr::from_bytes_until_nul(&buf).unwrap();

	func(str);
}

fn format_cstr_args<const N: usize, F>(args: Arguments<'_>, func: F)
where
	F: FnOnce(&CStr)
{
	format_cstr::<N, _, _>(
		|cursor| {
			let _ = cursor.write_fmt(args);
		},
		func
	);
}

#[allow(clippy::needless_pass_by_value)]
fn format_list<'a, const N: usize, I, T, S, F>(values: I, sep: S, func: F)
where
	I: IntoIterator<Item = &'a T>,
	T: Format + 'a,
	S: Format,
	F: FnOnce(&CStr)
{
	format_cstr::<N, _, _>(
		|cursor| {
			for (i, value) in values.into_iter().enumerate() {
				if i > 0 {
					sep.write(cursor);
				}

				value.write(cursor);
			}
		},
		func
	);
}

pub struct BufferSrc(FilterContext);

impl BufferSrc {
	/// # Safety
	/// frame contains raw pointers and must be valid
	pub unsafe fn send_frame(&mut self, frame: AVFrame) -> Result<()> {
		ffi!(
			av_buffersrc_add_frame,
			self.as_mut_ptr(),
			frame.as_mut_ptr()
		)?;

		drop(frame);

		Ok(())
	}

	pub fn drain(&mut self) -> Result<()> {
		ffi!(
			av_buffersrc_add_frame,
			self.as_mut_ptr(),
			MutPtr::null().as_mut_ptr()
		)?;

		Ok(())
	}
}

deref_inner!(BufferSrc, FilterContext);

pub struct BufferSink(FilterContext);

impl BufferSink {
	/// # Safety
	/// frame contains raw pointers and must be valid
	pub unsafe fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		ffi_optional!(
			av_buffersink_get_frame,
			self.as_mut_ptr(),
			frame.as_mut_ptr()
		)
	}

	pub fn set_frame_size(&mut self, frame_size: u32) {
		ffi!(av_buffersink_set_frame_size, self.as_mut_ptr(), frame_size);
	}
}

deref_inner!(BufferSink, FilterContext);

pub struct AudioBufferSrc(BufferSrc);

impl AudioBufferSrc {
	options! {
		time_base: Rational = c"time_base",
		sample_rate: i32 = c"sample_rate",
		sample_fmt: SampleFormat = c"sample_fmt",
		ch_layout_str: &CStr = c"channel_layout",
		channels: i32 = c"channels"
	}

	/// # Panics
	/// if the filter is not found
	pub fn new(graph: &mut FilterGraph) -> Self {
		#[allow(clippy::expect_used)]
		let filter = Filters::find_by_name_c(c"abuffer").expect("Filter not found");
		let ctx = graph.create_filter_c(filter, Some(c"abuffersrc"));

		Self(BufferSrc(ctx))
	}

	pub fn set_ch_layout(&mut self, layout: &ChannelLayout) {
		format_cstr::<2048, _, _>(
			|cursor| layout.write(cursor),
			|str| self.set_ch_layout_str(str)
		);
	}
}

deref_inner!(AudioBufferSrc, BufferSrc);

pub struct AudioBufferSink(BufferSink);

impl AudioBufferSink {
	options! {
		sample_fmts: &[SampleFormat] = c"sample_fmts",
		sample_rates: &[i32] = c"sample_rates",
		ch_layouts_str: &CStr = c"ch_layouts",
		all_channel_counts: bool = c"all_channel_counts"
	}

	/// # Panics
	/// if the filter is not found
	pub fn new(graph: &mut FilterGraph) -> Self {
		#[allow(clippy::expect_used)]
		let filter = Filters::find_by_name_c(c"abuffersink").expect("Filter not found");
		let ctx = graph.create_filter_c(filter, Some(c"abuffersink"));

		Self(BufferSink(ctx))
	}

	pub fn set_ch_layouts(&mut self, layouts: &[ChannelLayout]) {
		format_list::<2048, _, _, _, _>(layouts, "|", |str| self.set_ch_layouts_str(str));
	}
}

deref_inner!(AudioBufferSink, BufferSink);

pub struct Volume(FilterContext);

impl Volume {
	new_filter!(c"volume");

	options! {
		volume_str: &CStr = c"volume"
	}

	pub fn set_volume(&mut self, volume: f64) {
		format_cstr_args::<32, _>(format_args!("{}", volume), |volume| {
			self.set_volume_str(volume);
		});
	}
}

deref_inner!(Volume, FilterContext);

pub struct SetRate(FilterContext);

impl SetRate {
	new_filter!(c"asetrate");

	options! {
		sample_rate_signed: i32 = c"sample_rate"
	}

	/// # Panics
	/// if the sample rate cannot fit into an i32
	pub fn set_sample_rate(&mut self, rate: u32) {
		#[allow(clippy::unwrap_used)]
		self.set_sample_rate_signed(rate.try_into().unwrap());
	}
}

deref_inner!(SetRate, FilterContext);

pub struct Tempo(FilterContext);

impl Tempo {
	new_filter!(c"atempo");

	options! {
		tempo: f64 = c"tempo"
	}
}

deref_inner!(Tempo, FilterContext);

pub struct FirEqualizer(FilterContext);

impl FirEqualizer {
	new_filter!(c"firequalizer");

	options! {
		gain_entries_str: &CStr = c"gain_entry",
		delay: f64 = c"delay",
		accuracy: f64 = c"accuracy",
		multi_channel: bool = c"multi",
		zero_phase: bool = c"zero_phase",
		use_fft_two_channels: bool = c"fft2"
	}

	pub fn set_gain_entries(&mut self, entries: &[(f64, f64)]) {
		format_cstr::<1024, _, _>(
			|cursor| {
				for (i, (frequency, gain)) in entries.iter().enumerate() {
					if i > 0 {
						let _ = cursor.write(b"; ");
					}

					let _ = cursor.write_fmt(format_args!("entry({}, {})", frequency, gain));
				}
			},
			|str| self.set_gain_entries_str(str)
		);
	}
}

deref_inner!(FirEqualizer, FilterContext);

pub struct Resample(FilterContext);

impl Resample {
	new_filter!(c"aresample");

	options! {
		sample_rate_signed: i32 = c"sample_rate"
	}

	/// # Panics
	/// if the sample rate cannot fit into an i32
	pub fn set_sample_rate(&mut self, rate: u32) {
		#[allow(clippy::unwrap_used)]
		self.set_sample_rate_signed(rate.try_into().unwrap());
	}
}

deref_inner!(Resample, FilterContext);

pub struct Pulsator(FilterContext);

impl Pulsator {
	new_filter!(c"apulsator");

	options! {
		level_in: f64 = c"level_in",
		level_out: f64 = c"level_out",
		modulation: f64 = c"amount",
		offset_left: f64 = c"offset_l",
		offset_right: f64 = c"offset_r",
		width: f64 = c"width",
		frequency: f64 = c"hz"
	}
}

deref_inner!(Pulsator, FilterContext);

pub struct Echo(FilterContext);

impl Echo {
	new_filter!(c"aecho");

	options! {
		input_gain: f32 = c"in_gain",
		output_gain: f32 = c"out_gain",
		delays_str: &CStr = c"delays",
		decays_str: &CStr = c"decays"
	}

	pub fn set_delays(&mut self, delays: &[f32]) {
		format_list::<1024, _, _, _, _>(delays, "|", |delays| self.set_delays_str(delays));
	}

	pub fn set_decays(&mut self, decays: &[f32]) {
		format_list::<1024, _, _, _, _>(decays, "|", |decays| self.set_decays_str(decays));
	}
}

deref_inner!(Echo, FilterContext);
