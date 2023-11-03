use ffmpeg_sys_next::AV_NOPTS_VALUE;

mod demuxer;
pub use demuxer::*;
mod format;
pub use format::*;
mod packet;
pub use packet::*;
mod frame;
pub use frame::*;
pub mod rational;
pub use rational::Rational;
mod resource;
pub use resource::*;
mod pool;
pub use pool::*;
mod codec;
pub use codec::*;
mod buffer;
use buffer::*;
mod codecs;

mod reader;
use reader::*;

pub const UNKNOWN_TIMESTAMP: i64 = AV_NOPTS_VALUE;
