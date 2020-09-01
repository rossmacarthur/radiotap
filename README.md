# radiotap

[![Crates.io Version](https://img.shields.io/crates/v/radiotap.svg)](https://crates.io/crates/radiotap)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/radiotap)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/radiotap/build/master)](https://github.com/rossmacarthur/radiotap/actions?query=workflow%3Abuild)

A parser for the [radiotap](http://www.radiotap.org/) capture format.

## Getting started

Add to your project with

```sh
cargo add radiotap
```

or directly editing your `Cargo.toml`

```toml
[dependencies]
radiotap = "2"
```

See the documentation [here](https://docs.rs/radiotap).

## Usage

See [examples/](examples/) for more.

```rust
// a capture off the wire or from a pcap file
let capture = &[0, 0, 0xd, 0x0, 0x5, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x30, /* ... */ ];

// parse the radiotap header from the capture into a `Header` struct
let header: radiotap::Header = radiotap::parse(capture)?;

// get the length of the entire header
let length = header.length();

// Unpack the `rate` field if it is present
if let radiotap::Header { rate: Some(rate), .. } = header {
    assert_eq!(rate.to_mbps(), 24.0);
}

// using the length we can determine the rest of the capture
// i.e. IEEE 802.11 header and body
let rest = &capture[length..];
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
