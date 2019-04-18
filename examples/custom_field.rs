//! This example demonstrates how to implement a custom Radiotap field parser by
//! implementing the `field::Field` trait.

extern crate radiotap;
use radiotap::{field, Error, RadiotapIterator};

/// Our custom Antenna Signal struct
#[derive(Debug)]
struct MyAntennaSignal {
    value: i8,
}

impl field::Field for MyAntennaSignal {
    fn from_bytes(input: &[u8]) -> Result<MyAntennaSignal, Error> {
        Ok(MyAntennaSignal {
            value: input[0] as i8,
        })
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
            let signal: MyAntennaSignal = field::from_bytes(data).unwrap();
            println!("{:?}", signal);
        }
    }
}
