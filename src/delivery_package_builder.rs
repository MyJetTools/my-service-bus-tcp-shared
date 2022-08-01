use crate::{tcp_message_id, tcp_serializers::*};

pub fn init_delivery_package(
    payload: &mut Vec<u8>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: i64,
) -> usize {
    payload.push(tcp_message_id::NEW_MESSAGES);
    pascal_string::serialize(payload, topic_id);
    pascal_string::serialize(payload, queue_id);
    i64::serialize(payload, subscriber_id);

    let amount_offset = payload.len();
    i32::serialize(payload, 0);

    amount_offset
}

pub fn update_amount_of_messages(
    payload: &mut Vec<u8>,
    messages_count_position: usize,
    amount: i32,
) {
    let size = amount.to_le_bytes();
    let dest = &mut payload[messages_count_position..messages_count_position + 4];
    dest.copy_from_slice(size.as_slice());
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use my_service_bus_shared::MySbMessageContent;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;
    use crate::{PacketProtVer, TcpContract};

    #[tokio::test]
    async fn test_basic_usecase_v2() {
        const PROTOCOL_VERSION: i32 = 2;

        let version = PacketProtVer {
            packet_version: 1,
            protocol_version: PROTOCOL_VERSION,
        };

        let mut headers = HashMap::new();
        headers.insert("1".to_string(), "1".to_string());
        headers.insert("2".to_string(), "2".to_string());

        let msg1 = MySbMessageContent::new(
            1,
            vec![1, 1, 1],
            Some(headers),
            DateTimeAsMicroseconds::now(),
        );
        let msg2 = MySbMessageContent::new(2, vec![2, 2, 2], None, DateTimeAsMicroseconds::now());

        let mut payload = Vec::new();

        let amount_position = init_delivery_package(&mut payload, "test_topic", "test_queue", 15);

        crate::tcp_serializers::messages_to_deliver::serialize(&mut payload, &msg1, 1, &version);
        crate::tcp_serializers::messages_to_deliver::serialize(&mut payload, &msg2, 2, &version);

        update_amount_of_messages(&mut payload, amount_position, 2);

        let tcp_contract = TcpContract::Raw(payload);

        let result = convert_from_raw(tcp_contract, &version).await;

        if let TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            mut messages,
        } = result
        {
            assert_eq!("test_topic", topic_id);
            assert_eq!("test_queue", queue_id);
            assert_eq!(15, confirmation_id);
            assert_eq!(2, messages.len());

            let result_msg1 = messages.remove(0);

            assert_eq!(1, result_msg1.attempt_no);
            assert_eq!(msg1.content, result_msg1.content);
            assert_eq!(true, result_msg1.headers.is_none());

            let result_msg2 = messages.remove(0);

            assert_eq!(2, result_msg2.attempt_no);
            assert_eq!(msg2.content, result_msg2.content);
            assert_eq!(true, result_msg2.headers.is_none());
        } else {
            panic!("We should not be ere")
        }
    }

    #[tokio::test]
    async fn test_basic_usecase_v3() {
        const PROTOCOL_VERSION: i32 = 3;

        let version = PacketProtVer {
            packet_version: 1,
            protocol_version: PROTOCOL_VERSION,
        };

        let mut headers = HashMap::new();
        headers.insert("1".to_string(), "1".to_string());
        headers.insert("2".to_string(), "2".to_string());

        let msg1 = MySbMessageContent::new(
            1,
            vec![1, 1, 1],
            Some(headers),
            DateTimeAsMicroseconds::now(),
        );
        let msg2 = MySbMessageContent::new(2, vec![2, 2, 2], None, DateTimeAsMicroseconds::now());

        let mut payload = Vec::new();

        let amount_position = init_delivery_package(&mut payload, "test_topic", "test_queue", 15);

        crate::tcp_serializers::messages_to_deliver::serialize(&mut payload, &msg1, 1, &version);
        crate::tcp_serializers::messages_to_deliver::serialize(&mut payload, &msg2, 2, &version);

        update_amount_of_messages(&mut payload, amount_position, 2);

        let tcp_contract = TcpContract::Raw(payload);

        let result = convert_from_raw(tcp_contract, &version).await;

        if let TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            mut messages,
        } = result
        {
            assert_eq!("test_topic", topic_id);
            assert_eq!("test_queue", queue_id);
            assert_eq!(15, confirmation_id);
            assert_eq!(2, messages.len());

            let result_msg1 = messages.remove(0);

            assert_eq!(1, result_msg1.attempt_no);
            assert_eq!(msg1.content, result_msg1.content);
            assert_eq!(2, result_msg1.headers.unwrap().len());

            let result_msg2 = messages.remove(0);

            assert_eq!(2, result_msg2.attempt_no);
            assert_eq!(msg2.content, result_msg2.content);
            assert_eq!(true, result_msg2.headers.is_none());
        } else {
            panic!("We should not be ere")
        }
    }
}
