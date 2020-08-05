use std::io::Read;

pub use error::ParseError;
pub use error::Result;

mod error;
mod read_ext;

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
pub fn parse<T: Read>(mut reader: T) -> Result<OpusHeaders> {
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

    let comment_len = comment_pages.iter().map(|p| p.payload.len()).sum();
    let mut comment_bytes = Vec::with_capacity(comment_len);

    for mut page in comment_pages {
        comment_bytes.append(&mut page.payload);
    }

    let co = CommentHeader::parse(&comment_bytes[..])?;

    Ok(OpusHeaders { id, comments: co })
}
