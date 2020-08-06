use crate::error::{ParseError, Result};
use crate::read_ext::ReadExt;
use std::io::Read;

pub(crate) struct OggPage {
    pub version: u8,
    pub header_type: u8,
    pub granule_position: i64,
    pub bitstream_serial_number: u32,
    pub page_sequence_number: u32,
    pub crc_checksum: u32,
    pub page_segments: u8,
    pub segment_table: Vec<u8>, // contains the amount of bytes of payload: bytes = sum(segment_table_entries)
    pub payload: Vec<u8>,
}

impl OggPage {
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<OggPage> {
        // test ogg magic and read page
        if &reader.read_four_bytes()? != b"OggS" {
            return Err(ParseError::InvalidOggPage);
        }

        let version = reader.read_u8_le()?;
        let header_type = reader.read_u8_le()?;
        let granule_position = reader.read_i64_le()?;
        let bitstream_serial_number = reader.read_u32_le()?;
        let page_sequence_number = reader.read_u32_le()?;
        let crc_checksum = reader.read_u32_le()?;
        let page_segments = reader.read_u8_le()?;
        let segment_table = reader.read_byte_vec(page_segments as usize)?;

        let total_segments = segment_table.iter().map(|&b| b as usize).sum();
        let payload = reader.read_byte_vec(total_segments)?;

        Ok(OggPage {
            version,
            header_type,
            granule_position,
            bitstream_serial_number,
            page_sequence_number,
            crc_checksum,
            page_segments,
            segment_table,
            payload,
        })
    }
}
