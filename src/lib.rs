use std::io::Read;

pub use error::ParseError;
pub use error::Result;

use std::fs::File;
use std::path::Path;
use std::io::BufReader;

mod error;
mod read_ext;

mod opus_packets;

mod opus_header_structs;
pub use opus_header_structs::*;

mod ogg_page;
use ogg_page::*;
use opus_packets::OpusPackets;

#[cfg(test)]
mod tests;

const MAX_COMMENT_HEADER_LEN: u32 = 125_829_120; // as defined in section 5.2 of https://tools.ietf.org/html/rfc7845#section-5

/// Both headers contained in an opus file.
#[derive(Debug)]
pub struct OpusHeaders {
    pub id: IdentificationHeader,
    pub comments: CommentHeader,
}

/// Parses an opus file given by the path.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse_from_path<P: AsRef<Path>>(path: P) -> Result<OpusHeaders> {
    parse_from_file(&File::open(path)?)
}

/// Parses an opus file given by the file parameter.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse_from_file(file: &File) -> Result<OpusHeaders> {
    parse_from_read(BufReader::new(file))
}

/// Parses an opus file given by a reader.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse_from_read<T: Read>(mut reader: T) -> Result<OpusHeaders> {
    let first_ogg_page = OggPage::parse(&mut reader)?;

    let id = IdentificationHeader::parse(&first_ogg_page.payload[..])?;

    let mut comment_pages = vec![];
    let first_page = OggPage::parse(&mut reader)?;
    
    // used to make sure the payload does not exceed 120MB
    let mut comment_size: u32 = first_page.payload.len() as u32;
    
    comment_pages.push(first_page);
    
    // header 0x01 signals that the page is the continuation of a previous page
    loop {
        let next_page = OggPage::parse(&mut reader)?;
        if next_page.header_type == 0x01 {
            comment_size += next_page.payload.len() as u32;
            if comment_size > MAX_COMMENT_HEADER_LEN {
                return Err(error::ParseError::CommentHeaderTooLarge); // abort if we exceed the limit
            }
            comment_pages.push(next_page);
        } else {
            break;
        }
    }

    // the value of comment_len should be equal to comment_size and can thus be MAX_COMMENT_HEADER_LEN at maximum
    let comment_len = comment_pages.iter().map(|p| p.payload.len()).sum();
    
    // sanity check. The only way this can be triggered is if the previous code contains errors
    if comment_len as u32 != comment_size {
        return Err(error::ParseError::LengthMismatch);
    }

    // concatenate all payloads into the actual comment header
    let mut comment_bytes = Vec::with_capacity(comment_len);
    for mut page in comment_pages {
        comment_bytes.append(&mut page.payload);
    }

    let co = CommentHeader::parse(&comment_bytes[..], comment_len as u32)?;

    Ok(OpusHeaders { id, comments: co })
}

/// Parses an opus file given by the path.
/// Either returns the Opus Packets, or an error if anything goes wrong.
/// This should not panic.
pub fn get_opus_packets_from_path<P: AsRef<Path>>(path: P) -> Result<OpusPackets> {
    get_opus_payload_from_file(&File::open(path)?)
}

/// Parses an opus file given by the file parameter.
/// Either returns the Opus Packets, or an error if anything goes wrong.
/// This should not panic.
pub fn get_opus_payload_from_file(file: &File) -> Result<OpusPackets> {
    get_opus_payload_from_read(BufReader::new(file))
}

/// Parses an opus file given by a reader.
/// Either returns the Opus Packets, or an error if anything goes wrong.
/// This should not panic.
pub fn get_opus_payload_from_read<T: Read>(mut reader: T) -> Result<OpusPackets> {

    // parse and ignore the id header page. used to advance the reader
    let _first_ogg_page = OggPage::parse(&mut reader)?;
    // parse and ignore the comment header
    let _comment_ogg_page = OggPage::parse(&mut reader)?;

    // exhaust comment pages
    let first_opus_page = loop {
        let next_page = OggPage::parse(&mut reader)?;
        
        // if this bit is 0, we finished the comment header and next_page is the first page containing actual opus data
        if next_page.header_type & 0x2 == 0 {
            break next_page;
        }

        // if the stream ends without any data, return the empty data
        if next_page.header_type & 0x4 == 0 {
            return Ok(OpusPackets::default());
        }
    };

    let opus_packets = OpusPackets::parse(&mut reader, first_opus_page)?;

    Ok(opus_packets)
}
