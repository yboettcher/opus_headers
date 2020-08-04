use std::collections::HashMap;
use std::io::{self, Read};
use std::str;

/// Both headers contained in an opus file.
#[derive(Debug)]
pub struct OpusHeaders {
    pub id: IdentificationHeader,
    pub comments: CommentHeader,
}

/// The identification header.
#[derive(Debug)]
pub struct IdentificationHeader {
    pub version: u8,
    pub channel_count: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: i16,
    pub channel_mapping_family: u8,
    pub channel_mapping_table: Option<ChannelMappingTable>,
}

/// This part is optionally included in the IdentificationHeader
#[derive(Debug)]
pub struct ChannelMappingTable {
    pub stream_count: u8,
    pub coupled_stream_count: u8,
    pub channel_mapping: Vec<u8>,
}

/// The Comment header containing a vendor string and the user comments as a map.
#[derive(Debug)]
pub struct CommentHeader {
    pub vendor: String,
    pub user_comments: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Encoding(str::Utf8Error),
    InvalidOggPage,
    DidNotFindHeaders,
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<str::Utf8Error> for ParseError {
    fn from(e: str::Utf8Error) -> Self {
        Self::Encoding(e)
    }
}

struct OggPage {
    pub version: u8,
    pub header_type: u8,
    pub granule_position: i64,
    pub bitstream_serial_number: u32,
    pub page_sequence_number: u32,
    pub crc_checksum: u32,
    pub page_segments: u8,
    pub segment_table: Vec<u8>, // contains the amount of bytes of payload: bytes = sum(segment_table_entries)
    // payload: Vec<u8> // if we want to check the CRC, we should keep the original Payload
    pub payload: OggPayload,
}

pub enum OggPayload {
    IdentificationHeader(IdentificationHeader),
    CommentHeader(CommentHeader),
}

/// Parses a file given by a reader.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse<T: Read>(mut reader: T) -> Result<OpusHeaders, ParseError> {
    let mut ogg_magic = [0; 4];

    // test ogg magic and read first page
    reader.read_exact(&mut ogg_magic)?;
    if ogg_magic != [0x4f, 0x67, 0x67, 0x53] {
        return Err(ParseError::InvalidOggPage);
    }
    let first_ogg_page = parse_ogg_page(&mut reader)?;

    // test ogg magic and read second page
    reader.read_exact(&mut ogg_magic)?;
    if ogg_magic != [0x4f, 0x67, 0x67, 0x53] {
        return Err(ParseError::InvalidOggPage);
    }
    let second_ogg_page = parse_ogg_page(&mut reader)?;

    // test ogg magic after second page (sanity check)
    reader.read_exact(&mut ogg_magic)?;
    if ogg_magic != [0x4f, 0x67, 0x67, 0x53] {
        return Err(ParseError::InvalidOggPage);
    }

    if let (OggPayload::IdentificationHeader(id), OggPayload::CommentHeader(co)) =
        (first_ogg_page.payload, second_ogg_page.payload)
    {
        return Ok(OpusHeaders { id, comments: co });
    }

    return Err(ParseError::DidNotFindHeaders);
}

fn parse_ogg_page<T: Read>(mut reader: T) -> Result<OggPage, ParseError> {
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

    let mut opus_magic = [0; 8];
    reader.read_exact(&mut opus_magic)?;

    // first packet, parse the identification header
    if header_type == 0x02 && opus_magic == [0x4f, 0x70, 0x75, 0x73, 0x48, 0x65, 0x61, 0x64] {
        let identification_header = parse_identification_header(&mut reader)?;
        return Ok(OggPage {
            version,
            header_type,
            granule_position,
            bitstream_serial_number,
            page_sequence_number,
            crc_checksum,
            page_segments,
            segment_table,
            payload: OggPayload::IdentificationHeader(identification_header),
        });
    }

    // not the first packet -> second packet, parse the comment header
    if header_type == 0x00 && opus_magic == [0x4f, 0x70, 0x75, 0x73, 0x54, 0x61, 0x67, 0x73] {
        let comment_header = parse_comment_header(&mut reader)?;
        return Ok(OggPage {
            version,
            header_type,
            granule_position,
            bitstream_serial_number,
            page_sequence_number,
            crc_checksum,
            page_segments,
            segment_table,
            payload: OggPayload::CommentHeader(comment_header),
        });
    }

    return Err(ParseError::InvalidOggPage);
}

/// Parses an identification header.
/// Returns an err if anything goes wrong.
fn parse_identification_header<T: Read>(mut reader: T) -> Result<IdentificationHeader, ParseError> {
    let mut buf = [0; 4];
    let version = {
        reader.read_exact(&mut buf[0..1])?;
        buf[0]
    };
    let channel_count = {
        reader.read_exact(&mut buf[0..1])?;
        buf[0]
    };
    let pre_skip = u16::from_le_bytes({
        reader.read_exact(&mut buf[0..2])?;
        [buf[0], buf[1]]
    });
    let input_sample_rate = u32::from_le_bytes({
        reader.read_exact(&mut buf)?;
        buf
    });
    let output_gain = i16::from_le_bytes({
        reader.read_exact(&mut buf[0..2])?;
        [buf[0], buf[1]]
    });
    let channel_mapping_family = {
        reader.read_exact(&mut buf[0..1])?;
        buf[0]
    };

    let channel_mapping_table = if channel_mapping_family != 0 {
        Some(parse_channel_mapping_table(&mut reader)?)
    } else {
        None
    };

    Ok(IdentificationHeader {
        version,
        channel_count,
        pre_skip,
        input_sample_rate,
        output_gain,
        channel_mapping_family,
        channel_mapping_table,
    })
}

/// parses a channel mapping table.
/// returns an err if anything goes wrong.
fn parse_channel_mapping_table<T: Read>(mut reader: T) -> Result<ChannelMappingTable, ParseError> {
    let mut buf = [0; 1];
    let stream_count = {
        reader.read_exact(&mut buf)?;
        buf[0]
    };
    let coupled_stream_count = {
        reader.read_exact(&mut buf)?;
        buf[0]
    };
    let mut channel_mapping = vec![0; stream_count as usize];
    reader.read_exact(&mut channel_mapping)?;

    Ok(ChannelMappingTable {
        stream_count,
        coupled_stream_count,
        channel_mapping,
    })
}

/// parses the comment header.
/// returns an err if anything goes wrong.
/// if a comment cannot be split into two parts by splitting at '=', the comment is ignored
fn parse_comment_header<T: Read>(mut reader: T) -> Result<CommentHeader, ParseError> {
    let mut buf = [0; 4];

    let vlen = u32::from_le_bytes({
        reader.read_exact(&mut buf)?;
        buf
    });
    let mut vstr_buffer = vec![0; vlen as usize];
    reader.read_exact(&mut vstr_buffer)?;
    let vstr = std::str::from_utf8(&vstr_buffer)?;

    let mut comments = HashMap::new();
    let commentlistlen = u32::from_le_bytes({
        reader.read_exact(&mut buf)?;
        buf
    });

    for _i in 0..commentlistlen {
        let commentlen = u32::from_le_bytes({
            reader.read_exact(&mut buf)?;
            buf
        });
        let mut comment_buffer = vec![0; commentlen as usize];
        reader.read_exact(&mut comment_buffer)?;
        let commentstr = std::str::from_utf8(&comment_buffer)?;
        let parts: Vec<_> = commentstr.splitn(2, "=").collect();
        if parts.len() == 2 {
            comments.insert(parts[0].to_string(), parts[1].to_string());
        } // else? malformed comment?
    }

    Ok(CommentHeader {
        vendor: vstr.to_string(),
        user_comments: comments,
    })
}
