use std::io::SeekFrom;

use xx_core::{debug, error};
use xx_url::http::{get, Body, HttpRequest, StatusCode};
use xx_url::net::conn::IpStrategy;

use super::*;

#[errors]
pub enum HttpError {
	#[display("HTTP error {}", f0)]
	BadStatus(StatusCode)
}

struct HttpStream {
	request: HttpRequest,
	body: Body,
	position: u64,
	length: Option<u64>
}

#[asynchronous]
impl HttpStream {
	fn get_range(range: &str) -> Option<(u64, u64)> {
		let mut split = range.split_whitespace();

		if !split.next()?.eq_ignore_ascii_case("bytes") {
			return None;
		}

		let mut range_and_length = split.next()?.split('/');
		let start = range_and_length.next()?.split('-').next()?;

		Some((start.parse().ok()?, range_and_length.next()?.parse().ok()?))
	}

	async fn get_body_for(
		request: &mut HttpRequest, start: u64
	) -> Result<(Body, u64, Option<u64>)> {
		let mut position = 0;
		let mut length = None;

		request.header("Range", format!("bytes={}-", start).as_str());

		let response = HttpRequest::run(request).await?;

		if !response.status().is_success() {
			return Err(HttpError::BadStatus(response.status()).into());
		}

		#[allow(clippy::never_loop)]
		loop {
			let Some(range) = response.headers().get_str("Content-Range")? else {
				break;
			};

			let Some((pos, len)) = Self::get_range(range) else {
				break;
			};

			position = pos;
			length = Some(len);

			if pos == start {
				break;
			}

			return Err(FormatError::InvalidSeek(pos, start).into());
		}

		Ok((response.into_body(), position, length))
	}

	async fn new(url: &str, strategy: IpStrategy) -> Result<Self> {
		let mut request = get(url);

		request.set_strategy(strategy);

		let (body, position, length) = Self::get_body_for(&mut request, 0).await?;

		Ok(Self { request, body, position, length })
	}
}

#[asynchronous]
impl Read for HttpStream {
	async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		match self.body.read(buf).await {
			Ok(n) => {
				#[allow(clippy::arithmetic_side_effects)]
				(self.position += n as u64);

				return Ok(n);
			}

			Err(err) => {
				if err.is_interrupted() {
					return Err(err);
				}

				error!(target: &*self, "== Read from body failed, retrying ({:?})", err);
			}
		}

		let old_pos = self.position;

		debug!(target: &*self, "== Retrying stream at position = {}", self.position);

		self.seek(SeekFrom::Start(self.position)).await?;

		debug!(target: &*self, "== After seek, position = {}", self.position);

		if self.position != old_pos {
			return Err(FormatError::InvalidSeek(old_pos, self.position).into());
		}

		let result = self.body.read(buf).await;

		#[allow(clippy::arithmetic_side_effects)]
		if let Ok(n) = &result {
			self.position += *n as u64;
		}

		return result;
	}
}

#[asynchronous]
impl Seek for HttpStream {
	#[allow(clippy::unwrap_used)]
	async fn seek(&mut self, seek: SeekFrom) -> Result<u64> {
		let pos = match seek {
			SeekFrom::Current(pos) => self.position.checked_add_signed(pos).unwrap(),
			SeekFrom::Start(pos) => pos,
			SeekFrom::End(pos) => self.stream_len().await?.checked_add_signed(pos).unwrap()
		};

		let (body, position, length) = Self::get_body_for(&mut self.request, pos).await?;

		self.body = body;
		self.position = position;
		self.length = length;

		Ok(self.position)
	}

	fn stream_position_fast(&self) -> bool {
		true
	}

	async fn stream_position(&mut self) -> Result<u64> {
		Ok(self.position)
	}

	fn stream_len_fast(&self) -> bool {
		true
	}

	async fn stream_len(&mut self) -> Result<u64> {
		match self.length {
			Some(len) => Ok(len),
			None => return Err(fmt_error!("Unknown length"))
		}
	}
}

impl StreamImpl for HttpStream {
	fn seekable(&self) -> bool {
		true
	}
}

pub struct HttpResource {
	url: String,
	strategy: IpStrategy
}

impl HttpResource {
	#[must_use]
	#[allow(clippy::impl_trait_in_params)]
	pub fn new(url: impl Into<String>) -> Self {
		Self { url: url.into(), strategy: IpStrategy::Default }
	}

	pub fn set_strategy(&mut self, strategy: IpStrategy) -> &mut Self {
		self.strategy = strategy;
		self
	}
}

#[asynchronous]
impl ResourceImpl for HttpResource {
	async fn create_stream(&self) -> Result<Stream> {
		Ok(Box::new(HttpStream::new(&self.url, self.strategy).await?))
	}
}
