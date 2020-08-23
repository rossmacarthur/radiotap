use radiotap::{field, Header};

#[test]
fn fixture_0() {
    let bytes = include_bytes!("fixtures/0.bin");
    let Header { tsft, .. } = radiotap::parse(&bytes[..]).unwrap();
    assert_eq!(tsft.unwrap().into_inner(), 0x8877665544332211);
}

#[test]
fn fixture_00() {
    let bytes = include_bytes!("fixtures/00.bin");
    let mut tsfts: Vec<u64> = Vec::new();
    let mut iter = radiotap::Iter::new(&bytes[..]).unwrap().into_default();
    while let Some(kind) = iter.next().unwrap() {
        match kind {
            field::Type::Tsft => tsfts.push(iter.read(kind).unwrap()),
            kind => iter.skip(kind).unwrap(),
        }
    }
    assert_eq!(tsfts, vec![0x8877665544332211, 0x1100ffeeddccbbaa])
}

// #[test]
// fn fixture_1() {
//     let bytes = include_bytes!("fixtures/1.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }

// #[test]
// fn fixture_0fcs() {
//     let bytes = include_bytes!("fixtures/0fcs.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }

// #[test]
// fn fixture_0v0() {
//     let bytes = include_bytes!("fixtures/0v0.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }

// #[test]
// fn fixture_0v0_2() {
//     let bytes = include_bytes!("fixtures/0v0-2.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }

// #[test]
// fn fixture_0v0_3() {
//     let bytes = include_bytes!("fixtures/0v0-3.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }

// #[test]
// fn fixture_0v0_4() {
//     let bytes = include_bytes!("fixtures/0v0-4.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }

// #[test]
// fn fixture_malformed_vendor() -> anyhow::Result<()> {
//     let bytes = include_bytes!("fixtures/malformed-vendor.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }
// #[test]
// fn fixture_unparsed_vendor() {
//     let bytes = include_bytes!("fixtures/unparsed-vendor.bin");
//     let tap = radiotap::parse(&bytes[..]).unwrap();
//     println!("{:?}", tap);
//     panic!()
// }
