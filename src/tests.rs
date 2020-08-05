use std::path::Path;

use crate::error::ParseError;
use crate::parse_from_path;

#[test]
fn test_standard_file() {
    let path = Path::new("test/silence_standard.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments.user_comments;
    
    assert_eq!(comments.get("ALBUM").unwrap(), "name of album", "album parsed incorrectly");
    assert_eq!(comments.get("ARTIST").unwrap(), "artist_tag", "artist parsed incorrectly");
    assert_eq!(comments.get("COMMENT").unwrap(), "some random comment", "comment parsed incorrectly");
    assert_eq!(comments.get("TITLE").unwrap(), "tag_title", "title parsed incorrectly");
    // if these work, I assume the rest works too
    assert_eq!(comments.keys().len(), 9, "incorrect amount of tags found");
}

#[test]
fn test_file_without_comments() {
    let path = Path::new("test/silence_no_tags.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments.user_comments;
    
    assert_eq!(comments.keys().len(), 0);
}

#[test]
fn test_large_file() {
    let path = Path::new("test/silence_multi_page_tags.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments.user_comments;
    
    // I know that the lyrics comment contains 210_000 bytes
    // If we get anything else, that's wrong.
    assert_eq!(comments.get("LYRICS").unwrap().len(), 210_000);
}

#[test]
fn test_malformed_file_1() {
    let path = Path::new("test/silence_malformed_missing_magic.opus");
    let headers = parse_from_path(path);
    
    if let Err(ParseError::InvalidOggPage) = headers {
        return;
    }
    
    println!("{:#?}", headers.unwrap());
    
    panic!("this file should not be accepted");
}

#[test]
fn test_malformed_file_2() {
    let path = Path::new("test/silence_malformed_wrong_length_too_long.opus");
    let headers = parse_from_path(path);
    
    if let Err(ParseError::Io(_e)) = headers {
        return;
    }
    
    println!("{:#?}", headers.unwrap());
    
    panic!("this file should not be accepted");
}


#[test]
fn test_malformed_file_3() {
    let path = Path::new("test/silence_malformed_wrong_length_too_short.opus");
    let headers = parse_from_path(path);
    
    if let Err(ParseError::Io(_e)) = headers {
        return;
    }
    
    println!("{:#?}", headers.unwrap());
    
    panic!("this file should not be accepted");
}


#[test]
fn test_non_existing_file() {
    let path = Path::new("test/not_found.opus");
    let headers = parse_from_path(path);
    
    if let Err(ParseError::Io(_e)) = headers {
        return;
    }
    
    println!("{:#?}", headers.unwrap());
    
    panic!("this file should not be accepted");
}
