#![allow(unreachable_pub, clippy::module_name_repetitions)]

use xx_core::opt::hint::*;

use super::*;
use crate::{macros::ebml_define, Reader};

pub mod internal;
pub mod parse;
pub mod spec;
pub mod types;

use parse::*;
pub use types::*;

pub type EbmlId = u64;
pub const UNKNOWN_SIZE: u64 = u64::MAX;

pub enum VIntKind {
	Unsigned,
	Signed,
	Id,
	Size
}
