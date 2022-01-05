use std::collections::HashMap;

use my_service_bus_shared::MessageId;
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

#[derive(Debug, Clone)]
pub struct TcpContractMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub content: Vec<u8>,
}

impl TcpContractMessage {
    #[inline]
    pub async fn deserialize<TSocketReader: SocketReader>(
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
            headers: None,
            attempt_no,
            content,
        };

        Ok(result)
    }

    pub async fn deserialize_v3<TSocketReader: SocketReader>(
        socket_reader: &mut TSocketReader,
    ) -> Result<Self, ReadingTcpContractFail> {
        let id = socket_reader.read_i64().await?;

        let attempt_no = socket_reader.read_i32().await?;

        let headers =
            super::common_deserializers::deserealize_message_headers(socket_reader).await?;

        let content = socket_reader.read_byte_array().await?;

        let result = Self {
            id,
            headers,
            attempt_no,
            content,
        };

        Ok(result)
    }
}
