# radiotap

[![Crates.io Version](https://img.shields.io/crates/v/radiotap.svg?style=flat-square&color=blue)][crates]
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg?style=flat-square)][docs]
[![Build Status](https://img.shields.io/travis/rossmacarthur/radiotap/master.svg?style=flat-square)][travis]

A parser for the [radiotap](http://www.radiotap.org/) capture format.

## Getting started

Add to your project with

```bash
cargo add radiotap
```

or directly editing your `Cargo.toml`

```toml
[dependencies]
radiotap = "1"
```

See the documentation [here](https://docs.rs/radiotap).

## Example usage

See [examples/](examples/) for more.

The `Radiotap::from_bytes(&capture)` constructor will parse all present fields
into a Radiotap struct:

```rust
let capture = [
    0, 0, 56, 0, 107, 8, 52, 0, 185, 31, 155, 154, 0, 0, 0, 0, 20, 0, 124, 21, 64, 1, 213,
    166, 1, 0, 0, 0, 64, 1, 1, 0, 124, 21, 100, 34, 249, 1, 0, 0, 0, 0, 0, 0, 255, 1, 80,
    4, 115, 0, 0, 0, 1, 63, 0, 0
];

let radiotap = Radiotap::from_bytes(&capture).unwrap();
println!("{:?}", radiotap.vht);
```

If you just want to parse a few specific fields from the radiotap capture you
can create an iterator using `RadiotapIterator::from_bytes(&capture)`:

```rust
let capture = [
    0, 0, 56, 0, 107, 8, 52, 0, 185, 31, 155, 154, 0, 0, 0, 0, 20, 0, 124, 21, 64, 1, 213,
    166, 1, 0, 0, 0, 64, 1, 1, 0, 124, 21, 100, 34, 249, 1, 0, 0, 0, 0, 0, 0, 255, 1, 80,
    4, 115, 0, 0, 0, 1, 63, 0, 0
];

for element in RadiotapIterator::from_bytes(&capture).unwrap() {
    match element {
        Ok((field::Kind::VHT, data)) => {
            let vht: field::VHT = field::from_bytes(data).unwrap();
            println!("{:?}", vht);
        },
        _ => {}
    }
}
```

## License

This project is dual licensed under the Apache 2.0 License and the MIT License.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for more
details.

[crates]: https://crates.io/crates/radiotap
[docs]: https://docs.rs/radiotap
[travis]: https://travis-ci.org/rossmacarthur/radiotap
