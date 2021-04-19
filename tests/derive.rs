use radiotap::field::Field;

#[test]
fn basic_struct() {
    #[derive(Debug, PartialEq, Field)]
    #[field(align = 1, size = 3)]
    struct Test {
        a: u8,
        b: i16,
    }

    impl From<[u8; 3]> for Test {
        fn from(bytes: [u8; 3]) -> Self {
            Self {
                a: bytes[0],
                b: i16::from_le_bytes({
                    let ptr = bytes[1..3].as_ptr() as *const [u8; 2];
                    unsafe { *ptr }
                }),
            }
        }
    }

    assert_eq!(Test::from_bytes([0, 0, 0]), Test { a: 0, b: 0 });
}

#[test]
fn basic_new_type_struct() {
    #[derive(Debug, PartialEq, Field)]
    #[field(align = 1, size = 3)]
    struct Test(u8, i16);

    impl From<[u8; 3]> for Test {
        fn from(bytes: [u8; 3]) -> Self {
            Self(
                bytes[0],
                i16::from_le_bytes({
                    let ptr = bytes[1..3].as_ptr() as *const [u8; 2];
                    unsafe { *ptr }
                }),
            )
        }
    }

    assert_eq!(Test::from_bytes([0, 0, 0]), Test(0, 0));
}
