use super::*;

macro_rules! new {
	($struct:ident, $new:ident) => {
		impl $struct {
			#[allow(clippy::new_without_default)]
			pub fn new() -> Self {
				Self(alloc_with(|| ffi!($new)))
			}
		}
	};
}

pub(super) use new;

macro_rules! deref_inner {
	($struct:ident, $target:ident) => {
		impl Deref for $struct {
			type Target = $target;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl DerefMut for $struct {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	};
}

pub(super) use deref_inner;

macro_rules! ptr_deref {
	($struct:ident, $av:path) => {
		/// For internal use only. Changing random fields is unsafe
		impl Deref for $struct {
			type Target = $av;

			fn deref(&self) -> &Self::Target {
				/* Safety: the pointer is always valid */
				unsafe { self.0.as_ref() }
			}
		}

		/// For internal use only. Changing random fields is unsafe
		impl DerefMut for $struct {
			fn deref_mut(&mut self) -> &mut Self::Target {
				/* Safety: the pointer is always valid */
				unsafe { self.0.as_mut() }
			}
		}

		#[allow(dead_code)]
		impl $struct {
			pub const fn as_ptr(&self) -> *const $av {
				self.0.as_ptr()
			}

			pub const fn as_mut_ptr(&self) -> *mut $av {
				self.0.as_mut_ptr()
			}
		}

		/* Safety: wrapper types are Send */
		unsafe impl Send for $struct {}

		/* Safety: wrapper types are Sync */
		unsafe impl Sync for $struct {}
	};
}

pub(super) use ptr_deref;

macro_rules! drop {
	($struct:ident, $free:ident) => {
		impl Drop for $struct {
			fn drop(&mut self) {
				let mut ptr = self.as_mut_ptr();

				/* we own this pointer */
				ffi!($free, &mut ptr);
			}
		}
	};
}

pub(super) use drop;

macro_rules! av_wrapper {
	($struct:ident, $av:path, $free:ident) => {
		pub struct $struct(MutNonNull<$av>);

		ptr_deref!($struct, $av);
		drop!($struct, $free);
	};

	($struct:ident, $av:path, $free:ident, $new:ident) => {
		av_wrapper!($struct, $av, $free);
		new!($struct, $new);
	};
}

pub(super) use av_wrapper;

macro_rules! define_av_alias {
	(
		#[repr($repr:ty)]
		$(#$attrs: tt)*
		$vis: vis
		enum $name:ident
		$($rest: tt)*
	) => {
		#[repr($repr)]
		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
		$(#$attrs)*
		$vis enum $name $($rest)*
	};
}

pub(super) use define_av_alias;

macro_rules! define_av_alias_casts {
	(
		#[repr($repr:ty)]
		$(#$attrs: tt)*
		$vis: vis
		enum $name:ident = $av:ident
		$($rest: tt)*
	) => {
		define_av_alias! {
			#[repr($repr)]
			#[derive(Default, FromPrimitive)]
			$(#$attrs)*
			$vis enum $name $($rest)*
		}

		impl From<$repr> for $name {
			fn from(format: $repr) -> Self {
				paste! {
					Self::[< from_ $repr >](format).unwrap_or_default()
				}
			}
		}

		impl From<$name> for $av {
			fn from(value: $name) -> Self {
				/* Safety: same repr */
				unsafe { transmute(value) }
			}
		}

		impl From<$av> for $name {
			fn from(value: $av) -> Self {
				/* shared lib values may be non-exhaustive */
				Self::from(value as $repr)
			}
		}
	};
}

pub(super) use define_av_alias_casts;

macro_rules! single_option {
	($name:ident : $type:ty = $option:literal) => {
		paste! {
			#[allow(dead_code)]
			pub fn [< set_ $name >](&mut self, $name: $type) {
				/* Safety: set the option */
				unsafe { <$type>::set_c(&mut self.options(), $option, $name) }.unwrap()
			}
		}
	};
}

pub(super) use single_option;

macro_rules! options {
	{
		$($name:ident: $type:ty = $option:literal),*
	} => {
		$(single_option!($name: $type = $option);)*
	}
}

pub(super) use options;
