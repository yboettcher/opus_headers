# Opus Header Parsing Library

Parsing of Opus Headers according to the [specification](https://tools.ietf.org/html/rfc7845#section-5)
Simply supply anything that implements the std::io::Read trait to the parse function.

# Usage

```
use std::path::Path;
use opus_headers::parse_from_path; // or parse_from_read or parse_from_file

fn main() {
    let path = Path::new("/mnt/RamDisk/silence.opus");
    let headers = parse_from_path(path).unwrap();

    let comments = headers.comments.user_comments;
    for (tag, value) in &comments {
        println!("{}: {}", tag, value);
    }
}
```

# License

As most Rust source, this is library is dual licensed under the Apache 2.0 and MIT license.
