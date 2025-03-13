#![allow(unreachable_pub)]

use xx_core::opt::hint::*;

use super::*;
use crate::macros::ebml_define;
use crate::Reader;

pub mod internal;
pub mod parse;
pub mod spec;
pub mod types;

pub use types::*;

use self::parse::*;

pub type EbmlId = u64;
pub const UNKNOWN_SIZE: u64 = u64::MAX;

pub enum VIntKind {
	Unsigned,
	Signed,
	Id,
	Size
}
