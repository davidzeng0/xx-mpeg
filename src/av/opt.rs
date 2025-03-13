use std::marker::PhantomData;
use std::mem::size_of_val;

use super::*;

pub trait OptionSetter: Sized {
	/// # Safety
	/// object has a valid ptr
	unsafe fn set(object: &mut Object<'_>, option: &str, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { Self::set_c(object, &into_cstr(option), value) }
	}

	/// # Safety
	/// object has a valid ptr
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()>;
}

impl OptionSetter for bool {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { i64::set_c(object, option, value as i64) }
	}
}

impl OptionSetter for i32 {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { i64::set_c(object, option, value as i64) }
	}
}

impl OptionSetter for i64 {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_int,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for f32 {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { f64::set_c(object, option, value as f64) }
	}
}

impl OptionSetter for f64 {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_double,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for &CStr {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value.as_ptr(),
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for &str {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { <&CStr>::set_c(object, option, &into_cstr(value)) }
	}
}

impl OptionSetter for AVRational {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_q,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for Rational {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { AVRational::set_c(object, option, value.into()) }
	}
}

pub struct ImageSize(pub u32, pub u32);

impl OptionSetter for ImageSize {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		ffi!(
			av_opt_set_image_size,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value.0.try_into().unwrap(),
			value.1.try_into().unwrap(),
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl<T: Copy> OptionSetter for &[T] {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		ffi!(
			av_opt_set_bin,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value.as_ptr().cast(),
			size_of_val(value).try_into().unwrap(),
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for AVPixelFormat {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_pixel_fmt,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for PixelFormat {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { AVPixelFormat::set_c(object, option, value.into()) }
	}
}

impl OptionSetter for AVSampleFormat {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_sample_fmt,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for SampleFormat {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { AVSampleFormat::set_c(object, option, value.into()) }
	}
}

pub struct VideoRate(pub AVRational);

impl OptionSetter for VideoRate {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_video_rate,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value.0,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for &AVChannelLayout {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		ffi!(
			av_opt_set_chlayout,
			object.0.as_mut_ptr().cast(),
			option.as_ptr(),
			value,
			AV_OPT_SEARCH_CHILDREN
		)?;

		Ok(())
	}
}

impl OptionSetter for AVChannelLayout {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		#[allow(clippy::needless_borrows_for_generic_args)]
		(unsafe { <&Self>::set_c(object, option, &value) })
	}
}

impl OptionSetter for &ChannelLayout {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { AVChannelLayout::set_c(object, option, value.into()) }
	}
}

impl OptionSetter for ChannelLayout {
	unsafe fn set_c(object: &mut Object<'_>, option: &CStr, value: Self) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { <&Self>::set_c(object, option, &value) }
	}
}

pub struct Object<'a>(MutNonNull<()>, PhantomData<&'a ()>);

impl Object<'_> {
	pub const fn from<T: ?Sized>(ptr: MutNonNull<T>) -> Self {
		Self(ptr.cast(), PhantomData)
	}

	/// # Safety
	/// the pointer passed to this `Object` is valid
	pub unsafe fn set<T: OptionSetter>(&mut self, option: &str, value: T) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { T::set(self, option, value) }
	}

	/// # Safety
	/// the pointer passed to this `Object` is valid
	pub unsafe fn set_c<T: OptionSetter>(&mut self, option: &CStr, value: T) -> Result<()> {
		/* Safety: guaranteed by caller */
		unsafe { T::set_c(self, option, value) }
	}
}
