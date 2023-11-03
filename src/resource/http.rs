use std::io::SeekFrom;

use xx_url::{
	http::{get, Body, HttpRequest},
	net::connection::IpStrategy
};

use super::*;

struct HttpStream {
	request: HttpRequest,
	body: Body,
	position: u64,
	length: Option<u64>
}

#[async_fn]
impl HttpStream {
	fn get_range(range: &String) -> Option<(u64, u64)> {
		let mut split = range.split_whitespace();

		if !split.next()?.eq_ignore_ascii_case("bytes") {
			return None;
		}

		let mut range_and_length = split.next()?.split('/');
		let start = range_and_length.next()?.split('-').next()?;

		Some((
			u64::from_str_radix(start, 10).ok()?,
			u64::from_str_radix(range_and_length.next()?, 10).ok()?
		))
	}

	async fn get_body_for(
		request: &mut HttpRequest, start: u64
	) -> Result<(Body, u64, Option<u64>)> {
		let mut position = 0;
		let mut length = None;

		request.header("Range", format!("bytes={}-", start));

		let response = HttpRequest::run(request).await?;

		loop {
			let Some(range) = response.headers().get("content-range") else {
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

			return Err(Error::new(
				ErrorKind::InvalidData,
				format!(
					"Server returned content starting at {}, requested for {}",
					pos, start
				)
			));
		}

		Ok((response.into_body(), position, length))
	}

	async fn new(url: &str, strategy: IpStrategy) -> Result<HttpStream> {
		let mut request = get(url)?;

		request.set_strategy(strategy);

		let (body, position, length) = Self::get_body_for(&mut request, 0).await?;

		Ok(Self { request, body, position, length })
	}
}

impl Read for HttpStream {
	read_wrapper! {
		inner = body;
		mut inner = body;
	}
}

#[async_trait_impl]
impl Seek for HttpStream {
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

	async fn stream_position(&mut self) -> Result<u64> {
		Err(Error::new(ErrorKind::Other, "Cannot use stream_position"))
	}

	async fn stream_len(&mut self) -> Result<u64> {
		match self.length {
			Some(len) => Ok(len),
			None => return Err(Error::new(ErrorKind::Other, "Unknown length"))
		}
	}
}

impl StreamTrait for HttpStream {}

pub struct HttpResource {
	url: String,
	strategy: IpStrategy
}

impl HttpResource {
	pub fn new(url: &str) -> Self {
		Self {
			url: url.to_string(),
			strategy: IpStrategy::Default
		}
	}

	pub fn set_strategy(&mut self, strategy: IpStrategy) -> &mut Self {
		self.strategy = strategy;
		self
	}
}

#[async_trait_impl]
impl ResourceTrait for HttpResource {
	async fn create_stream(&self) -> Result<Stream> {
		Ok(Box::new(HttpStream::new(&self.url, self.strategy).await?))
	}
}
