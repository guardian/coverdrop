use std::io;

pub trait ReadExt: io::Read {
    fn read_byte(&mut self) -> io::Result<u8>;
}

impl<T> ReadExt for T
where
    T: io::Read,
{
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}
