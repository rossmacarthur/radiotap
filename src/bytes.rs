use std::io::{Cursor, Read, Result, Seek, SeekFrom};

pub trait Advance {
    fn advance(&mut self, advance: usize) -> Result<()>;
}

pub trait Align {
    fn align(&mut self, align: usize) -> Result<()>;
}

pub trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<T> Advance for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn advance(&mut self, advance: usize) -> Result<()> {
        self.seek(SeekFrom::Current(advance as i64)).map(drop)
    }
}

impl<T> Align for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn align(&mut self, align: usize) -> Result<()> {
        let align = align as u64;
        assert!(matches!(align, 1 | 2 | 4 | 8 | 16));
        let new_pos = (self.position() + align - 1) & !(align - 1);
        self.seek(SeekFrom::Start(new_pos)).map(drop)
    }
}

impl<R> ReadExt for R where R: Read {}
