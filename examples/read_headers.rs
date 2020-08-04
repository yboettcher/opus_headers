extern crate opus_parse;

use std::io::BufReader;
use std::fs::File;
use opus_parse::parse;

fn main() {
    let f = File::open(format!("{}/examples/silence.opus", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let mut reader = BufReader::new(f);
    
    
    let headers = parse(&mut reader).unwrap();
    println!("{:?}", headers);
} 
