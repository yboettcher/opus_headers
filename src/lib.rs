use std::io::Read;

pub use error::ParseError;
pub use error::Result;

use std::fs::File;
use std::path::Path;
use std::io::BufReader;

mod error;
mod read_ext;

mod opus_header_structs;
use opus_header_structs::*;

mod ogg_page;
use ogg_page::*;

#[cfg(test)]
mod tests;

/// Both headers contained in an opus file.
#[derive(Debug)]
pub struct OpusHeaders {
    pub id: IdentificationHeader,
    pub comments: CommentHeader,
}

pub fn parse_from_path<P: AsRef<Path>>(path: P) -> Result<OpusHeaders> {
    parse_from_file(&File::open(path)?)
}

pub fn parse_from_file(file: &File) -> Result<OpusHeaders> {
    parse_from_read(BufReader::new(file))
}

/// Parses a file given by a reader.
/// Either returns the Opus Headers, or an error if anything goes wrong.
/// This should not panic.
pub fn parse_from_read<T: Read>(mut reader: T) -> Result<OpusHeaders> {
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
