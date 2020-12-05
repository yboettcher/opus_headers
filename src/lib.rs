use std::io::Read;

pub use error::ParseError;
pub use error::Result;

use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::Seek;

use ogg::PacketReader;
pub use ogg::Packet;

mod error;
mod read_ext;

mod opus_header_structs;
pub use opus_header_structs::*;

#[cfg(test)]
mod tests;

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
pub fn parse_from_read<T: Read + Seek>(reader: T) -> Result<OpusHeaders> {
    let mut packet_reader = PacketReader::new(reader);
    
    let first_ogg_page = packet_reader.read_packet()?.ok_or(error::ParseError::UnexpectedEndOfStream)?;

    let id = IdentificationHeader::parse(&first_ogg_page.data[..])?;

    let comment_packet = packet_reader.read_packet()?.ok_or(error::ParseError::UnexpectedEndOfStream)?;

    let co = CommentHeader::parse(&comment_packet.data[..], comment_packet.data.len() as u32)?;

    Ok(OpusHeaders { id, comments: co })
}

/// Parses an opus file given by the path.
/// Either returns the Opus Packets, or an error if anything goes wrong.
/// This should not panic.
pub fn get_opus_packets_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<Packet>> {
    get_opus_payload_from_file(&File::open(path)?)
}

/// Parses an opus file given by the file parameter.
/// Either returns the Opus Packets, or an error if anything goes wrong.
/// This should not panic.
pub fn get_opus_payload_from_file(file: &File) -> Result<Vec<Packet>> {
    get_opus_payload_from_read(BufReader::new(file))
}

/// Parses an opus file given by a reader.
/// Either returns the Opus Packets, or an error if anything goes wrong.
/// This should not panic.
pub fn get_opus_payload_from_read<T: Read + Seek>(reader: T) -> Result<Vec<Packet>> {
    let mut packet_reader = PacketReader::new(reader);
    // parse and ignore the id header packet.
    let _first_ogg_page = packet_reader.read_packet()?.ok_or(error::ParseError::UnexpectedEndOfStream)?;
    // parse and ignore the comment header packet
    let _comment_ogg_page = packet_reader.read_packet()?.ok_or(error::ParseError::UnexpectedEndOfStream)?;

    let mut opus_packets = vec![];
    while let Some(packet) = packet_reader.read_packet()? {
        opus_packets.push(packet);
    }

    Ok(opus_packets)
}
