use my_service_bus_shared::MessageId;
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

#[derive(Debug, Clone)]
pub struct TcpContractMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

impl TcpContractMessage {
    #[inline]
    pub async fn serialize<TSocketReader: SocketReader>(
        socket_reader: &mut TSocketReader,
        packet_version: i32,
    ) -> Result<Self, ReadingTcpContractFail> {
        let id = socket_reader.read_i64().await?;

        let attempt_no = if packet_version == 1 {
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
