use enumflags2::{bitflags, BitFlag, BitFlags};
use xx_core::error::*;
use xx_pulse::*;

mod av;
mod codec;
pub mod codecs;
pub mod demuxer;
pub mod errors;
pub mod filter;
pub mod format;
pub mod frame;
mod macros;
pub mod packet;
pub mod rational;
mod reader;

use self::demuxer::*;
use self::reader::*;
pub mod resource;
pub use av::UNKNOWN_TIMESTAMP;
pub use codec::*;
pub use errors::*;
pub use format::*;
pub use frame::*;
pub use packet::*;
pub use rational::*;
pub use resource::*;

extern crate self as xx_mpeg;

pub extern crate constcat;
