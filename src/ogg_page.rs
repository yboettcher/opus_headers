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
        let mut ogg_magic = [0; 4];

        // test ogg magic and read page
        reader.read_exact(&mut ogg_magic)?;
        if &ogg_magic != b"OggS" {
            return Err(ParseError::InvalidOggPage);
        }

        let version = reader.read_u8_le()?;
        let header_type = reader.read_u8_le()?;
        let granule_position = reader.read_i64_le()?;
        let bitstream_serial_number = reader.read_u32_le()?;
        let page_sequence_number = reader.read_u32_le()?;
        let crc_checksum = reader.read_u32_le()?;
        let page_segments = reader.read_u8_le()?;
        let mut segment_table_bytes = vec![0; page_segments as usize];
        let segment_table = {
            reader.read_exact(&mut segment_table_bytes)?;
            segment_table_bytes
        };

        let total_segments = segment_table.iter().map(|&b| b as usize).sum();
        let mut payload = vec![0; total_segments];

        reader.read_exact(&mut payload)?;

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
