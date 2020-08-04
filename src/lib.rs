use std::collections::HashMap;
use std::error::Error;
use std::io::Read;

mod conversion;
mod errors;

use conversion::*;
use errors::*;

#[derive(Debug)]
pub struct OpusHeaders {
    pub id: IdentificationHeader,
    pub comments: CommentHeader,
}

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

#[derive(Debug)]
pub struct ChannelMappingTable {
    pub stream_count: u8,
    pub coupled_stream_count: u8,
    pub channel_mapping: Vec<u8>,
}

#[derive(Debug)]
pub struct CommentHeader {
    pub vendor: String,
    pub user_comments: HashMap<String, String>,
}

/// Reads the given amount of bytes from the Reader and returns them as Vec<u8>
/// If anything goes wrong (reader error, amount of read bytes does not equal amount of requested bytes) an err is returned.
/// The returned Vec always has exactly 'bytes' items.
fn read_bytes<T: Read>(mut reader: T, bytes: usize) -> Result<Vec<u8>, Box<dyn Error>> { // const generics would be great... we would not need vec anymore 
    let mut arr = vec![0; bytes];
    let got = reader.read(&mut arr)?;
    if bytes != got {
        Err(Box::new(DidNotReadEnough { expected: bytes, got}))
    } else {
        Ok(arr)
    }
}

/// Parses a file given by a reader.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse<T: Read> (mut reader: T) -> Result<OpusHeaders, Box<dyn Error>> {
    
    let mut ident: Option<IdentificationHeader> = None;
    let mut comment: Option<CommentHeader> = None;
    
    while let Ok(cursor) = read_bytes(&mut reader, 1) {
        let cursor = cursor[0];
        let mut match_result = matches_head(cursor, &mut reader)?;
        while let OpusHeadsMatch::Retry(current) = match_result {
            match_result = matches_head(current, &mut reader)?;
        }
        
        match match_result {
            OpusHeadsMatch::Header => {
                ident = Some(parse_identification_header(&mut reader)?);
            }
            OpusHeadsMatch::Tags => {
                comment = Some(parse_comment_header(&mut reader)?);
            }
            _ => {
            
            }
        }
        
        if ident.is_some() && comment.is_some() {
            break;
        }
    }
    
    if let (Some(id), Some(co)) = (ident, comment) {
        Ok(OpusHeaders{ id, comments: co})
    } else {
        Err(Box::new(DidNotFindBothHeaders))
    }
    
}

/// Parses an identification header.
/// Returns an err if anything goes wrong.
fn parse_identification_header<T: Read>(mut reader: T) -> Result<IdentificationHeader, Box<dyn Error>> {
    let version = read_bytes(&mut reader, 1)?[0];
    let channel_count = read_bytes(&mut reader, 1)?[0];
    let pre_skip = to_u16(&read_bytes(&mut reader, 2)?);
    let input_sample_rate = to_u32(&read_bytes(&mut reader, 4)?);
    let output_gain = to_i16(&read_bytes(&mut reader, 2)?);
    let channel_mapping_family = read_bytes(&mut reader, 1)?[0];

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
        channel_mapping_table
    })
}

/// parses a channel mapping table.
/// returns an err if anything goes wrong.
fn parse_channel_mapping_table<T: Read>(mut reader: T) -> Result<ChannelMappingTable, Box<dyn Error>> {
    let stream_count = read_bytes(&mut reader, 1)?[0];
    let coupled_stream_count = read_bytes(&mut reader, 1)?[0];
    let channel_mapping = read_bytes(&mut reader, stream_count as usize)?;
    
    Ok(ChannelMappingTable {
        stream_count,
        coupled_stream_count,
        channel_mapping
    })
}

/// parses the comment header.
/// returns an err if anything goes wrong.
/// if a comment cannot be split into two parts by splitting at '=', the comment is ignored
fn parse_comment_header<T: Read>(mut reader: T) -> Result<CommentHeader, Box<dyn Error>> {
    let vlen = to_u32(&read_bytes(&mut reader, 4)?);
    let vstr_bytes = read_bytes(&mut reader, vlen as usize)?;
    let vstr = std::str::from_utf8(&vstr_bytes)?;

    let mut comments = HashMap::new();
    let commentlistlen = to_u32(&read_bytes(&mut reader, 4)?);

    for _i in 0..commentlistlen {
        let commentlen = to_u32(&read_bytes(&mut reader, 4)?);
        let commentstr = read_bytes(&mut reader, commentlen as usize)?;
        let commentstr = std::str::from_utf8(&commentstr)?;
        let parts: Vec<_> = commentstr.splitn(2, "=").collect();
        if parts.len() == 2 {
            comments.insert(parts[0].to_string(), parts[1].to_string());
        } // else? malformed comment?
    }
    
    Ok(CommentHeader {
        vendor: vstr.to_string(),
        user_comments: comments
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
    Retry(u8)
}

// incrementally parses the magic numbers of the identification and comment header.
// if any byte does not match, we either return none, as this is clearly not any header, or, if the byte is 0x4f, we return that byte (which is why we always have to save it in a 'next' variable) and tell the caller to try again
fn matches_head<T: Read>(current: u8, mut reader: T) -> Result<OpusHeadsMatch, Box<dyn Error>> {
    let mut next = 0;
    if current == 0x4f {
        next = read_bytes(&mut reader, 1)?[0];
        if next == 0x70 {
            next = read_bytes(&mut reader, 1)?[0];
            if next == 0x75 {
                next = read_bytes(&mut reader, 1)?[0];
                if next == 0x73 {
                    next = read_bytes(&mut reader, 1)?[0];
                    if next == 0x48 {
                        next = read_bytes(&mut reader, 1)?[0];
                        if next == 0x65 {
                            next = read_bytes(&mut reader, 1)?[0];
                            if next == 0x61 {
                                next = read_bytes(&mut reader, 1)?[0];
                                if next == 0x64 {
                                    return Ok(OpusHeadsMatch::Ident);
                                }
                            }
                        }
                    } else {
                        if next == 0x54 {
                            next = read_bytes(&mut reader, 1)?[0];
                            if next == 0x61 {
                                next = read_bytes(&mut reader, 1)?[0];
                                if next == 0x67 {
                                    next = read_bytes(&mut reader, 1)?[0];
                                    if next == 0x73 {
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
    if next == 0x4f {
        return Ok(OpusHeadsMatch::Retry(next));
    }
    return Ok(OpusHeadsMatch::None);
}
