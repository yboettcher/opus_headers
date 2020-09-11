use std::io::{Read, Result};

pub trait ReadExt {
    fn read_u8_le(&mut self) -> Result<u8>;
    fn read_i8_le(&mut self) -> Result<i8>;
    fn read_u16_le(&mut self) -> Result<u16>;
    fn read_i16_le(&mut self) -> Result<i16>;
    fn read_u32_le(&mut self) -> Result<u32>;
    fn read_i32_le(&mut self) -> Result<i32>;
    fn read_u64_le(&mut self) -> Result<u64>;
    fn read_i64_le(&mut self) -> Result<i64>;
    fn read_byte_vec(&mut self, amount: usize) -> Result<Vec<u8>>;
    fn read_four_bytes(&mut self) -> Result<[u8; 4]>;
    fn read_eight_bytes(&mut self) -> Result<[u8; 8]>;
}

impl<T> ReadExt for T where T: Read {
    fn read_u8_le(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(u8::from_le_bytes(buf))
    }

    fn read_i8_le(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(i8::from_le_bytes(buf))
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_i16_le(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i32_le(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_u64_le(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_i64_le(&mut self) -> Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    // note that this function allocates the given amount of Bytes before actually reading anything.
    // Thus, one needs to be careful to not exhaust the computers memory by passing a very large 'amount' parameter
    fn read_byte_vec(&mut self, amount: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; amount];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
    
    fn read_four_bytes(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
    
    fn read_eight_bytes(&mut self) -> Result<[u8; 8]> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}
