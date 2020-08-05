extern crate opus_headers;

use opus_headers::parse;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let f = File::open(format!(
        "{}/examples/silence_with_long_comments.opus",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let mut reader = BufReader::new(f);

    let headers = parse(&mut reader).unwrap();
    println!("{:#?}", headers);
}
