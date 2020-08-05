use crate::parse_error::*;
use std::collections::HashMap;
use std::io::Read;
use std::str;

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

impl IdentificationHeader {
    /// Parses an identification header.
    /// Returns an err if anything goes wrong.
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<IdentificationHeader, ParseError> {
        // check magic
        let mut opus_magic = [0; 8];
        reader.read_exact(&mut opus_magic)?;
        if opus_magic != [0x4f, 0x70, 0x75, 0x73, 0x48, 0x65, 0x61, 0x64] {
            return Err(ParseError::InvalidOpusHeader);
        }

        // parse header
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
            Some(ChannelMappingTable::parse(&mut reader)?)
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
}

impl ChannelMappingTable {
    /// parses a channel mapping table.
    /// returns an err if anything goes wrong.
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<ChannelMappingTable, ParseError> {
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
}

impl CommentHeader {
    /// parses the comment header.
    /// returns an err if anything goes wrong.
    /// if a comment cannot be split into two parts by splitting at '=', the comment is ignored
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<CommentHeader, ParseError> {
        // check magic
        let mut opus_magic = [0; 8];
        reader.read_exact(&mut opus_magic)?;
        if opus_magic != [0x4f, 0x70, 0x75, 0x73, 0x54, 0x61, 0x67, 0x73] {
            return Err(ParseError::InvalidOpusHeader);
        }

        let mut buf = [0; 4];

        let vlen = u32::from_le_bytes({
            reader.read_exact(&mut buf)?;
            buf
        });
        let mut vstr_buffer = vec![0; vlen as usize];
        reader.read_exact(&mut vstr_buffer)?;
        let vstr = str::from_utf8(&vstr_buffer)?;

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
            let commentstr = str::from_utf8(&comment_buffer)?;
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
}
