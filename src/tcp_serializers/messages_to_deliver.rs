use my_service_bus_shared::MySbMessageContent;
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

use crate::MessageToDeliverTcpContract;

pub fn serialize(
    dest: &mut Vec<u8>,
    msg: &MySbMessageContent,
    attempt_no: i32,
    protocol_version: i32,
    packet_version: i32,
) {
    if protocol_version < 3 {
        serialize_v2(dest, msg, attempt_no, packet_version);
    } else {
        serialize_v3(dest, msg, attempt_no);
    }
}

pub fn serialize_v2(
    dest: &mut Vec<u8>,
    msg: &MySbMessageContent,
    attempt_no: i32,
    packet_version: i32,
) {
    crate::tcp_serializers::i64::serialize(dest, msg.id);

    if packet_version == 1 {
        crate::tcp_serializers::i32::serialize(dest, attempt_no);
    }
    super::byte_array::serialize(dest, msg.content.as_slice());
}

pub fn serialize_v3(dest: &mut Vec<u8>, msg: &MySbMessageContent, attempt_no: i32) {
    crate::tcp_serializers::i64::serialize(dest, msg.id);
    crate::tcp_serializers::i32::serialize(dest, attempt_no);
    super::message_headers::serialize(dest, msg.headers.as_ref());
    super::byte_array::serialize(dest, msg.content.as_slice());
}

pub async fn deserialize<TSocketReader: SocketReader>(
    socket_reader: &mut TSocketReader,
    packet_version: i32,
    protocol_version: i32,
) -> Result<MessageToDeliverTcpContract, ReadingTcpContractFail> {
    if protocol_version < 3 {
        return deserialize_v2(socket_reader, packet_version).await;
    }

    return deserialize_v3(socket_reader).await;
}

pub async fn deserialize_v2<TSocketReader: SocketReader>(
    socket_reader: &mut TSocketReader,
    packet_version: i32,
) -> Result<MessageToDeliverTcpContract, ReadingTcpContractFail> {
    let id = socket_reader.read_i64().await?;

    let attempt_no = if packet_version == 1 {
        socket_reader.read_i32().await?
    } else {
        0
    };

    let content = socket_reader.read_byte_array().await?;

    let result = MessageToDeliverTcpContract {
        id,
        headers: None,
        attempt_no,
        content,
    };

    Ok(result)
}

pub async fn deserialize_v3<TSocketReader: SocketReader>(
    socket_reader: &mut TSocketReader,
) -> Result<MessageToDeliverTcpContract, ReadingTcpContractFail> {
    let id = socket_reader.read_i64().await?;

    let attempt_no = socket_reader.read_i32().await?;

    let headers = crate::tcp_serializers::message_headers::deserialize(socket_reader).await?;

    let content = socket_reader.read_byte_array().await?;

    let result = MessageToDeliverTcpContract {
        id,
        headers,
        attempt_no,
        content,
    };

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use my_service_bus_shared::MySbMessageContent;
    use my_tcp_sockets::socket_reader::SocketReaderMock;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    #[tokio::test]
    pub async fn test_v2() {
        const PROTOCOL_VERSION: i32 = 1;
        const PACKET_VERSION: i32 = 2;

        let mut headers = HashMap::new();
        headers.insert("key1".to_string(), "value1".to_string());

        let src_msg = MySbMessageContent {
            id: 1,
            time: DateTimeAsMicroseconds::now(),
            content: vec![0u8, 1u8, 2u8],
            headers: Some(headers),
        };

        let mut serialized_data = Vec::new();

        super::serialize(
            &mut serialized_data,
            &src_msg,
            1,
            PROTOCOL_VERSION,
            PACKET_VERSION,
        );

        let mut socket_reader = SocketReaderMock::new();
        socket_reader.push(serialized_data.as_slice());

        let result = super::deserialize(&mut socket_reader, PACKET_VERSION, PROTOCOL_VERSION)
            .await
            .unwrap();

        assert_eq!(src_msg.id, result.id);
        assert_eq!(src_msg.content, result.content);
        assert_eq!(true, result.headers.is_none());
    }

    #[tokio::test]
    pub async fn test_v3() {
        const PROTOCOL_VERSION: i32 = 3;
        const PACKET_VERSION: i32 = 2;

        let mut headers = HashMap::new();
        headers.insert("key1".to_string(), "value1".to_string());

        let src_msg = MySbMessageContent {
            id: 1,
            time: DateTimeAsMicroseconds::now(),
            content: vec![0u8, 1u8, 2u8],
            headers: Some(headers),
        };

        let mut serialized_data = Vec::new();

        super::serialize(
            &mut serialized_data,
            &src_msg,
            1,
            PROTOCOL_VERSION,
            PACKET_VERSION,
        );

        let mut socket_reader = SocketReaderMock::new();
        socket_reader.push(serialized_data.as_slice());

        let result = super::deserialize(&mut socket_reader, PACKET_VERSION, PROTOCOL_VERSION)
            .await
            .unwrap();

        assert_eq!(src_msg.id, result.id);
        assert_eq!(src_msg.content, result.content);

        let headers = result.headers.unwrap();
        assert_eq!(1, headers.len());

        assert_eq!("value1", headers.get("key1").unwrap());
    }
}
