use std::io::Read;
use crate::ogg_page::OggPage;
use crate::error::Result;

#[derive(Debug, Default)]
pub struct OpusPackets(pub Vec<OpusPacket>);
#[derive(Debug, Default)]
pub struct OpusPacket(pub Vec<u8>);

impl OpusPacket {
	/// Takes the first page of an opus packet and a reader. Reads until either the reader ends, or any page is not a continuation of the previous page.
	/// returns the parsed opus packet and either the first page of the next packet, iff there are any, or None, if the reader ended
	pub(crate) fn parse<T: Read>(mut reader: T, first_page: OggPage) -> Result<(Self, Option<OggPage>)> {
		let mut continue_reading = first_page.header_type & 0x4 == 0;

		let mut relevant_pages = vec![first_page];

		let mut next_packet_page = None;
		
		while continue_reading {
			let next_page = OggPage::parse(&mut reader)?;

			let is_relevant = next_page.header_type & 0x1 == 1;
			let is_last = next_page.header_type & 0x4 == 0;
			continue_reading = is_relevant && !is_last;

			if is_relevant {
				relevant_pages.push(next_page);
			} else {
				// in this case, is_relevant is false => continue_reading is also false and the loop ends
				next_packet_page = Some(next_page);
			}
		}

		let bytes = relevant_pages.drain(..).map(|page| page.payload).fold(vec![], |mut bytes, mut payload| {
			bytes.append(&mut payload);
			bytes
		});
		
		Ok((Self(bytes), next_packet_page))
	}
}

impl OpusPackets {
	pub(crate) fn parse<T: Read>(mut reader: T, first_page: OggPage) -> Result<OpusPackets> {
		let mut packets = vec![];

		let mut next_page = first_page;
		
		loop {
			let (packet, next) = OpusPacket::parse(&mut reader, next_page)?;

			packets.push(packet);

			if let Some(next) = next {
				next_page = next;
			} else {
				break;
			}
		}
		Ok(Self(packets))
	}
}
