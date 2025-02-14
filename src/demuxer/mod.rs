pub(crate) mod av;
pub mod mkv;

use super::*;

#[asynchronous(impl(mut, box))]
pub trait DemuxerImpl {
	async fn open(&mut self, data: &mut FormatData) -> Result<()>;

	async fn seek(
		&mut self, data: &mut FormatData, track: u32, timecode: u64, flags: BitFlags<SeekFlag>
	) -> Result<()>;

	async fn read_packet(&mut self, data: &mut FormatData, packet: &mut Packet) -> Result<bool>;
}

pub type Demuxer = Box<dyn DemuxerImpl + Send + Sync>;

#[asynchronous(impl(mut, box))]
pub trait DemuxerClassImpl: Send + Sync {
	fn name(&self) -> &'static str;

	async fn create(&self, reader: Reader) -> Result<Demuxer>;

	async fn probe(&self, reader: &mut Reader) -> Result<f32>;
}

pub type DemuxerClass<'a> = &'a dyn DemuxerClassImpl;
