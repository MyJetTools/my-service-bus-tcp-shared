#[cfg(test)]
use super::TSocketReader;
#[cfg(test)]
use crate::ReadingTcpContractFail;
#[cfg(test)]
use async_trait::async_trait;

#[cfg(test)]
pub struct DataReaderMock {
    data: Vec<u8>,
}
#[cfg(test)]
impl DataReaderMock {
    pub fn new() -> DataReaderMock {
        DataReaderMock { data: Vec::new() }
    }

    pub fn push(&mut self, data: &[u8]) {
        self.data.extend(data);
    }
}

#[cfg(test)]
#[async_trait]
impl TSocketReader for DataReaderMock {
    async fn read_byte(&mut self) -> Result<u8, ReadingTcpContractFail> {
        let result = self.data.remove(0);
        Ok(result)
    }

    async fn read_i32(&mut self) -> Result<i32, ReadingTcpContractFail> {
        const DATA_SIZE: usize = 4;

        let mut buf = [0u8; DATA_SIZE];

        buf.copy_from_slice(&self.data[0..DATA_SIZE]);

        let result = i32::from_le_bytes(buf);

        for _ in 0..DATA_SIZE {
            self.data.remove(0);
        }

        Ok(result)
    }

    async fn read_bool(&mut self) -> Result<bool, ReadingTcpContractFail> {
        let result = self.read_byte().await?;
        Ok(result > 0u8)
    }

    async fn read_byte_array(&mut self) -> Result<Vec<u8>, ReadingTcpContractFail> {
        let len = self.read_i32().await? as usize;

        let mut result: Vec<u8> = Vec::new();

        for b in self.data.drain(0..len) {
            result.push(b);
        }

        Ok(result)
    }

    async fn read_buf(&mut self, buf: &mut [u8]) -> Result<(), ReadingTcpContractFail> {
        buf.copy_from_slice(self.data.drain(0..buf.len()).as_slice());
        Ok(())
    }

    async fn read_i64(&mut self) -> Result<i64, ReadingTcpContractFail> {
        const DATA_SIZE: usize = 8;

        let mut buf = [0u8; DATA_SIZE];

        buf.copy_from_slice(&self.data[0..DATA_SIZE]);

        let result = i64::from_le_bytes(buf);

        for _ in 0..DATA_SIZE {
            self.data.remove(0);
        }

        Ok(result)
    }
}
