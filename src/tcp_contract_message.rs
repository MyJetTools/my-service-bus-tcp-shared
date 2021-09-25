use my_service_bus_shared::MessageId;

use crate::{PacketProtVer, ReadingTcpContractFail, TSocketReader};

#[derive(Debug, Clone)]
pub struct TcpContractMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

impl TcpContractMessage {
    #[inline]
    pub async fn serialize<TGSocketReader: TSocketReader>(
        socket_reader: &mut TGSocketReader,
        ver: &PacketProtVer,
    ) -> Result<Self, ReadingTcpContractFail> {
        let id = crate::legacy::read_long(socket_reader, ver).await?;

        let attempt_no = if ver.packet_version == 1 {
            socket_reader.read_i32().await?
        } else {
            0
        };

        let content = socket_reader.read_byte_array().await?;

        let result = Self {
            id,
            attempt_no,
            content,
        };

        Ok(result)
    }
}
