# radiotap

A parser for the [Radiotap](http://www.radiotap.org/) capture format.

## Usage

The `Radiotap::from_bytes(&capture)` constructor will parse all present fields into a Radiotap
struct:

```rust
extern crate radiotap;
use radiotap::{Radiotap};

fn main() {
    let capture = [
        0, 0, 56, 0, 107, 8, 52, 0, 185, 31, 155, 154, 0, 0, 0, 0, 20, 0, 124, 21, 64, 1, 213,
        166, 1, 0, 0, 0, 64, 1, 1, 0, 124, 21, 100, 34, 249, 1, 0, 0, 0, 0, 0, 0, 255, 1, 80,
        4, 115, 0, 0, 0, 1, 63, 0, 0
    ];

    let radiotap = Radiotap::from_bytes(&capture).unwrap();
    println!("{:?}", radiotap);
}
```

If you just want to parse a few specific fields from the Radiotap capture you can create an
iterator using `RadiotapIterator::from_bytes(&capture)`:

```rust
extern crate radiotap;
use radiotap::{RadiotapIterator, field};

fn main() {
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
}
```

See [examples/](examples/) for more.

# License

This project is dual licensed under the Apache 2.0 License and the MIT License. See the
[LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files.
