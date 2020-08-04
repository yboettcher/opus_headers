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

/// Parses a file given by a reader.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse<T: Read>(mut reader: T) -> Result<OpusHeaders, ParseError> {
    let mut ident: Option<IdentificationHeader> = None;
    let mut comment: Option<CommentHeader> = None;

    let mut cursor = [0; 1];

    while let Ok(()) = reader.read_exact(&mut cursor) {
        let cursor_value = cursor[0];
        let mut match_result = matches_head(cursor_value, &mut reader)?;
        while let OpusHeadsMatch::Retry(current) = match_result {
            match_result = matches_head(current, &mut reader)?;
        }

        match match_result {
            OpusHeadsMatch::Ident => {
                ident = Some(parse_identification_header(&mut reader)?);
            }
            OpusHeadsMatch::Comment => {
                comment = Some(parse_comment_header(&mut reader)?);
            }
            _ => {}
        }

        if ident.is_some() && comment.is_some() {
            break;
        }
    }

    if let (Some(id), Some(co)) = (ident, comment) {
        Ok(OpusHeaders { id, comments: co })
    } else {
        Err(ParseError::DidNotFindHeaders)
    }
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
        let commentstr =
            std::str::from_utf8(&comment_buffer)?;
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

/// Used to signal the result of the head match
/// None: This does not match any head,
/// Ident: This matches the identification header magic number
/// Comment: This matches the comment header magic number
/// Retry(u8): We found another 0x4f byte. In this case, we did not find any header, but the 0x4f might be the start of the actual header.
enum OpusHeadsMatch {
    None,
    Ident,
    Comment,
    Retry(u8),
}

/// incrementally parses the magic numbers of the identification and comment header.
/// if any byte does not match, we either return none, as this is clearly not any header, or, if the byte is 0x4f, we return that byte (which is why we always have to save it in a 'next' variable) and tell the caller to try again
fn matches_head<T: Read>(current: u8, mut reader: T) -> Result<OpusHeadsMatch, ParseError> {
    // There is probably a dozen better ways to do this, but this works
    let mut next = [0; 1];
    if current == 0x4f {
        reader.read_exact(&mut next)?;
        if next[0] == 0x70 {
            reader.read_exact(&mut next)?;
            if next[0] == 0x75 {
                reader.read_exact(&mut next)?;
                if next[0] == 0x73 {
                    reader.read_exact(&mut next)?;
                    if next[0] == 0x48 {
                        reader.read_exact(&mut next)?;
                        if next[0] == 0x65 {
                            reader.read_exact(&mut next)?;
                            if next[0] == 0x61 {
                                reader.read_exact(&mut next)?;
                                if next[0] == 0x64 {
                                    return Ok(OpusHeadsMatch::Ident);
                                }
                            }
                        }
                    } else {
                        if next[0] == 0x54 {
                            reader.read_exact(&mut next)?;
                            if next[0] == 0x61 {
                                reader.read_exact(&mut next)?;
                                if next[0] == 0x67 {
                                    reader.read_exact(&mut next)?;
                                    if next[0] == 0x73 {
                                        return Ok(OpusHeadsMatch::Comment);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if next[0] == 0x4f {
        return Ok(OpusHeadsMatch::Retry(next[0]));
    }
    return Ok(OpusHeadsMatch::None);
}
