use std::io::Read;
use std::result;

pub use error::ParseError;

use read_ext::ReadExt;

mod error;
mod read_ext;

/// A specialized [`Result`][std::result::Result] type for the fallible functions.
pub type Result<T, E = ParseError> = result::Result<T, E>;

mod opus_header_structs;
use opus_header_structs::*;

mod ogg_page;
use ogg_page::*;

/// Both headers contained in an opus file.
#[derive(Debug)]
pub struct OpusHeaders {
    pub id: IdentificationHeader,
    pub comments: CommentHeader,
}

/// Parses a file given by a reader.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse<T: Read>(mut reader: T) -> Result<OpusHeaders, ParseError> {
    let first_ogg_page = OggPage::parse(&mut reader)?;

    let id = IdentificationHeader::parse(&first_ogg_page.payload[..])?;

    let mut comment_pages = vec![];
    comment_pages.push(OggPage::parse(&mut reader)?);

    // header 0x01 signals that the page is the continuation of a previous page
    loop {
        let next_page = OggPage::parse(&mut reader)?;
        if next_page.header_type == 0x01 {
            comment_pages.push(next_page);
        } else {
            break;
        }
    }

    let mut comment_bytes: Vec<u8> = vec![];

    for mut page in comment_pages {
        comment_bytes.append(&mut page.payload);
    }

    let co = CommentHeader::parse(&comment_bytes[..])?;

    Ok(OpusHeaders { id, comments: co })
}
