use radiotap::field::{Field, FromArray};

#[test]
fn derive_basic_struct() {
    #[derive(Debug, PartialEq, FromArray)]
    struct SubTest(i8);

    #[derive(Debug, PartialEq, Field, FromArray)]
    #[field(align = 1, size = 6)]
    struct Test {
        a: u8,
        b: i16,
        c: [u8; 2],
        d: (),
        #[field(size = 1)]
        e: SubTest,
    }

    assert_eq!(
        Test::from_bytes([1, 2, 3, 4, 5, 6]),
        Test {
            a: 1,
            b: 0x0302,
            c: [4, 5],
            d: (),
            e: SubTest(6)
        }
    );
}

#[test]
fn derive_new_type_struct() {
    #[derive(Debug, PartialEq, FromArray)]
    struct SubTest(i8);

    #[derive(Debug, PartialEq, Field, FromArray)]
    #[field(align = 1, size = 6)]
    struct Test(u8, i16, [u8; 2], (), #[field(size = 1)] SubTest);

    assert_eq!(
        Test::from_bytes([1, 2, 3, 4, 5, 6]),
        Test(1, 0x0302, [4, 5], (), SubTest(6))
    );
}
