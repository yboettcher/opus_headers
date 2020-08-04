# Opus Header Parsing Library

Parsing of Opus Headers according to the [specification](https://tools.ietf.org/html/rfc7845#section-5)
Simply supply anything that implements the std::io::Read trait to the parse function.
There is a usage example that can be run with cargo run --example read_headers.

# License

As this library was created to allow opus support in [polaris](https://github.com/agersant/polaris), it uses the same License: MIT.
Maybe I will add Apache too.
