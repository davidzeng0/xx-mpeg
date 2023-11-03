mod mkv;
pub use mkv::*;
use xx_core::error::*;
use xx_pulse::*;

use super::*;

#[async_trait]
pub trait DemuxerTrait {
	async fn open(&mut self, context: &mut FormatContext) -> Result<()>;

	async fn seek(&mut self, track: u32, timecode: u64) -> Result<()>;

	async fn read_packet(
		&mut self, context: &mut FormatContext, packet: &mut Packet, pool: Option<&Pool>
	) -> Result<bool>;
}

pub type Demuxer = Box<dyn DemuxerTrait>;

#[async_trait]
pub trait DemuxerClassTrait: Send + Sync {
	fn name(&self) -> &'static str;

	async fn create(&self, reader: Reader) -> Result<Demuxer>;

	async fn probe(&self, reader: &mut Reader) -> Result<f32>;
}

pub type DemuxerClass<'a> = &'a dyn DemuxerClassTrait;
