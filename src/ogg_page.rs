use crate::opus_header_structs::*;
use crate::parse_error::*;
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
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<OggPage, ParseError> {
        let mut ogg_magic = [0; 4];

        // test ogg magic and read page
        reader.read_exact(&mut ogg_magic)?;
        if ogg_magic != [0x4f, 0x67, 0x67, 0x53] {
            return Err(ParseError::InvalidOggPage);
        }

        let mut buf = [0; 8];
        let version = {
            reader.read_exact(&mut buf[0..1])?;
            buf[0]
        };
        let header_type = {
            reader.read_exact(&mut buf[0..1])?;
            buf[0]
        };
        let granule_position = i64::from_le_bytes({
            reader.read_exact(&mut buf[0..8])?;
            buf
        });
        let bitstream_serial_number = u32::from_le_bytes({
            reader.read_exact(&mut buf[0..4])?;
            [buf[0], buf[1], buf[2], buf[3]]
        });
        let page_sequence_number = u32::from_le_bytes({
            reader.read_exact(&mut buf[0..4])?;
            [buf[0], buf[1], buf[2], buf[3]]
        });
        let crc_checksum = u32::from_le_bytes({
            reader.read_exact(&mut buf[0..4])?;
            [buf[0], buf[1], buf[2], buf[3]]
        });
        let page_segments = {
            reader.read_exact(&mut buf[0..1])?;
            buf[0]
        };
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
