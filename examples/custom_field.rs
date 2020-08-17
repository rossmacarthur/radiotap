//! This example demonstrates how to implement a custom radiotap field parser by
//! implementing the `field::Field` trait.

use radiotap::{field, Error, RadiotapIterator};

/// Our custom Antenna Signal struct
#[derive(Debug)]
struct MyAntennaSignal(i8);

impl MyAntennaSignal {
    fn new(input: &[u8]) -> Result<MyAntennaSignal, Error> {
        Ok(MyAntennaSignal(input[0] as i8))
    }
}

fn main() {
    let capture = [
        0, 0, 56, 0, 107, 8, 52, 0, 185, 31, 155, 154, 0, 0, 0, 0, 20, 0, 124, 21, 64, 1, 213, 166,
        1, 0, 0, 0, 64, 1, 1, 0, 124, 21, 100, 34, 249, 1, 0, 0, 0, 0, 0, 0, 255, 1, 80, 4, 115, 0,
        0, 0, 1, 63, 0, 0,
    ];

    for element in RadiotapIterator::from_bytes(&capture).unwrap() {
        if let Ok((field::Kind::AntennaSignal, data)) = element {
            let signal = MyAntennaSignal::new(data).unwrap();
            println!("{:?}", signal);
        }
    }
}
