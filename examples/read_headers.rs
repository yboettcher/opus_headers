extern crate opus_headers;

use opus_headers::parse_from_read;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let f = File::open(format!(
        "{}/examples/silence.opus",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let mut reader = BufReader::new(f);

    let headers = parse_from_read(&mut reader).unwrap();
    println!("{:#?}", headers);
}
