use std::path::Path;
use opus_headers::parse_from_path; // or parse_from_read or parse_from_file

fn main() {
    let path = Path::new("examples/silence.opus");
    let headers = parse_from_path(path).unwrap();

    let comments = headers.comments.user_comments;
    for (tag, value) in &comments {
        println!("{}: {}", tag, value);
    }
}
