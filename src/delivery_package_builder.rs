use my_service_bus_shared::{queue_with_intervals::QueueWithIntervals, MySbMessageContent};

use crate::{common_serializers::*, tcp_message_id, TcpContract};

pub struct DeliveryPackageBuilder<'s> {
    pub topic_id: &'s str,
    pub queue_id: &'s str,
    pub subscriber_id: i64,
    pub delivery_packet_version: i32,
    pub messages: Vec<(&'s MySbMessageContent, i32)>,
    pub ids: QueueWithIntervals,
    pub payload_size: usize,
}

impl<'s> DeliveryPackageBuilder<'s> {
    pub fn new(
        topic_id: &'s str,
        queue_id: &'s str,
        subscriber_id: i64,
        delivery_packet_version: i32,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            subscriber_id,
            delivery_packet_version,
            messages: Vec::new(),
            ids: QueueWithIntervals::new(),
            payload_size: 0,
        }
    }

    pub fn add_message(&mut self, msg: &'s MySbMessageContent, attempt_no: i32) {
        self.payload_size += msg.content.len();
        self.messages.push((msg, attempt_no));
        self.ids.enqueue(msg.id);
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn build(&self) -> TcpContract {
        let mut buffer = Vec::new();

        buffer.push(tcp_message_id::NEW_MESSAGES);
        serialize_pascal_string(&mut buffer, self.topic_id);
        serialize_pascal_string(&mut buffer, self.queue_id);
        serialize_i64(&mut buffer, self.subscriber_id);

        self.serialize_messages(&mut buffer);

        TcpContract::NewMessagesServerSide(buffer)
    }

    fn serialize_messages(&self, result: &mut Vec<u8>) {
        let messages_count = self.messages.len() as i32;

        serialize_i32(result, messages_count);

        for (msg_content, attempt_no) in &self.messages {
            serialize_message(
                result,
                msg_content,
                *attempt_no,
                self.delivery_packet_version,
            );
        }
    }
}

fn serialize_message(
    dest: &mut Vec<u8>,
    msg: &MySbMessageContent,
    attempt_no: i32,
    packet_version: i32,
) {
    crate::common_serializers::serialize_i64(dest, msg.id);

    if packet_version == 1 {
        serialize_i32(dest, attempt_no);
    }
    serialize_byte_array(dest, msg.content.as_slice());
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use my_tcp_sockets::socket_reader::SocketReaderMock;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;
    use crate::ConnectionAttributes;

    #[tokio::test]
    async fn test_basic_usecase() {
        let contents = vec![
            MySbMessageContent::new(1, vec![1, 1, 1], DateTimeAsMicroseconds::now()),
            MySbMessageContent::new(2, vec![2, 2, 2], DateTimeAsMicroseconds::now()),
        ];

        let mut package_builder = DeliveryPackageBuilder::new("test_topic", "test_queue", 15, 1);

        package_builder.add_message(contents.get(0).unwrap(), 1);
        package_builder.add_message(contents.get(1).unwrap(), 2);

        let tcp_contract = package_builder.build();

        let payload = tcp_contract.serialize();

        let mut socket_reader = SocketReaderMock::new();

        let mut attr = ConnectionAttributes::new();
        let mut versions = HashMap::new();
        versions.insert(tcp_message_id::NEW_MESSAGES, 1);
        attr.versions.update(&versions);
        socket_reader.push(&payload);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

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

            let msg1 = messages.remove(0);

            assert_eq!(1, msg1.attempt_no);
            assert_eq!(vec![1, 1, 1], msg1.content);
        } else {
            panic!("We should not be ere")
        }
    }
}
