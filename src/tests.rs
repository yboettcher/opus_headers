use std::path::Path;

use crate::error::ParseError;
use crate::parse_from_path;

#[test]
fn test_standard_file() {
    let path = Path::new("test/silence_standard.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments;

    assert_eq!(comments.get_value("ALBUM").unwrap(), "name of album", "album parsed incorrectly");
    assert_eq!(comments.get_value("ARTIST").unwrap(), "artist_tag", "artist parsed incorrectly");
    assert_eq!(comments.get_value("COMMENT").unwrap(), "some random comment", "comment parsed incorrectly");
    assert_eq!(comments.get_value("TITLE").unwrap(), "tag_title", "title parsed incorrectly");
    // if these work, I assume the rest works too
    assert_eq!(comments.user_comments.len(), 9, "incorrect amount of tags found");
}

#[test]
fn test_multivalue_file() {
    let path = Path::new("test/silence_multivalue.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments;
    assert_eq!(
        comments.get_values("ARTIST").unwrap(),
        ["artist_tag", "other_artist"],
        "artist parsed incorrectly"
    );
    assert_eq!(
        comments.get_values("GENRE").unwrap(),
        ["silence", "quiet"],
        "genre parsed incorrectly"
    );
}

#[test]
fn test_file_without_comments() {
    let path = Path::new("test/silence_no_tags.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments;

    assert_eq!(comments.user_comments.len(), 0);
    assert!(comments.get_value("ARTIST").is_none());
    assert!(comments.get_values("ARTIST").is_none());
}

#[test]
fn test_large_file() {
    let path = Path::new("test/silence_multi_page_tags.opus");
    let headers = parse_from_path(path).unwrap();
    let comments = headers.comments;

    // I know that the lyrics comment contains 210_000 bytes
    // If we get anything else, that's wrong.
    assert_eq!(comments.get_value("LYRICS").unwrap().len(), 210_000);
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

// These are now covered by the logic implemented for the malicious oom file
/*
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
*/

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

#[test]
fn test_malicious_file() {
    let path = Path::new("test/malicious_oom.opus");
    let headers = parse_from_path(path);

    if let Err(ParseError::CommentTooLong) = headers {
        return;
    }

    println!("{:#?}", headers.unwrap());

    panic!("this file should not be accepted");
}
