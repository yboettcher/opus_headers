use crate::error::{ParseError, Result};
use crate::read_ext::ReadExt;
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
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<IdentificationHeader> {
        // check magic
        let mut opus_magic = [0; 8];
        reader.read_exact(&mut opus_magic)?;
        if &opus_magic != b"OpusHead" {
            return Err(ParseError::InvalidOpusHeader);
        }

        // parse header
        let version = reader.read_u8_le()?;
        let channel_count = reader.read_u8_le()?;
        let pre_skip = reader.read_u16_le()?;
        let input_sample_rate = reader.read_u32_le()?;
        let output_gain = reader.read_i16_le()?;
        let channel_mapping_family = reader.read_u8_le()?;

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
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<ChannelMappingTable> {
        let stream_count = reader.read_u8_le()?;
        let coupled_stream_count = reader.read_u8_le()?;
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
    pub(crate) fn parse<T: Read>(mut reader: T) -> Result<CommentHeader> {
        // check magic
        let mut opus_magic = [0; 8];
        reader.read_exact(&mut opus_magic)?;
        if &opus_magic != b"OpusTags" {
            return Err(ParseError::InvalidOpusHeader);
        }

        let vlen = reader.read_u32_le()?;
        let mut vstr_buffer = vec![0; vlen as usize];
        reader.read_exact(&mut vstr_buffer)?;
        let vstr = str::from_utf8(&vstr_buffer)?;

        let mut comments = HashMap::new();
        let commentlistlen = reader.read_u32_le()?;

        for _i in 0..commentlistlen {
            let commentlen = reader.read_u32_le()?;
            let mut comment_buffer = vec![0; commentlen as usize];
            reader.read_exact(&mut comment_buffer)?;
            let commentstr = str::from_utf8(&comment_buffer)?;
            let parts: Vec<_> = commentstr.splitn(2, '=').collect();
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
