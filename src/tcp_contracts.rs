use my_service_bus_shared::{queue::TopicQueueType, queue_with_intervals::QueueIndexRange};
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

use crate::{ConnectionAttributes, TcpContractMessage};

use super::{common_serializers::*, tcp_message_id::*};

use std::collections::HashMap;

pub type RequestId = i64;

pub type ConfirmationId = i64;

#[derive(Debug)]
pub enum TcpContract {
    Ping,
    Pong,
    Greeting {
        name: String,
        protocol_version: i32,
    },
    Publish {
        topic_id: String,
        request_id: RequestId,
        persist_immediately: bool,
        data_to_publish: Vec<Vec<u8>>,
    },
    PublishResponse {
        request_id: RequestId,
    },
    Subscribe {
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    },
    SubscribeResponse {
        topic_id: String,
        queue_id: String,
    },
    NewMessagesServerSide(Vec<u8>),
    NewMessages {
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        messages: Vec<TcpContractMessage>,
    },
    NewMessagesConfirmation {
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },
    CreateTopicIfNotExists {
        topic_id: String,
    },
    IntermediaryConfirm {
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange>,
    },
    PacketVersions {
        packet_versions: HashMap<u8, i32>,
    },
    Reject {
        message: String,
    },
    AllMessagesConfirmedAsFail {
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },

    ConfirmSomeMessagesAsOk {
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange>,
    },
}

impl TcpContract {
    pub async fn deserialize<TSocketReader: SocketReader>(
        socket_reader: &mut TSocketReader,
        attr: &ConnectionAttributes,
    ) -> Result<TcpContract, ReadingTcpContractFail> {
        let packet_no = socket_reader.read_byte().await?;

        let result = match packet_no {
            PING => Ok(TcpContract::Ping {}),
            PONG => Ok(TcpContract::Pong {}),
            GREETING => {
                let name = super::common_deserializers::read_pascal_string(socket_reader).await?;
                let protocol_version = socket_reader.read_i32().await?;

                let result = TcpContract::Greeting {
                    name,
                    protocol_version,
                };
                Ok(result)
            }
            PUBLISH => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let request_id = socket_reader.read_i64().await?;
                let messages_count = socket_reader.read_i32().await? as usize;

                let mut data_to_publish: Vec<Vec<u8>> = Vec::with_capacity(messages_count);

                for _ in 0..messages_count {
                    let byte_array = socket_reader.read_byte_array().await?;
                    data_to_publish.push(byte_array);
                }

                let result = TcpContract::Publish {
                    topic_id,
                    request_id,
                    data_to_publish,
                    persist_immediately: socket_reader.read_bool().await?,
                };
                Ok(result)
            }
            PUBLISH_RESPONSE => {
                let request_id = socket_reader.read_i64().await?;
                let result = TcpContract::PublishResponse { request_id };

                Ok(result)
            }
            SUBSCRIBE => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_type = socket_reader.read_byte().await?;

                let queue_type = TopicQueueType::from_u8(queue_type);

                let result = TcpContract::Subscribe {
                    topic_id,
                    queue_id,
                    queue_type,
                };

                Ok(result)
            }
            SUBSCRIBE_RESPONSE => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let result = TcpContract::SubscribeResponse { topic_id, queue_id };

                Ok(result)
            }

            NEW_MESSAGES => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let records_len = socket_reader.read_i32().await? as usize;

                let mut messages: Vec<TcpContractMessage> = Vec::with_capacity(records_len);
                let packet_version = attr.get_packet_version(packet_no);
                for _ in 0..records_len {
                    let msg = TcpContractMessage::serialize(socket_reader, packet_version).await?;
                    messages.push(msg);
                }

                let result = TcpContract::NewMessages {
                    topic_id,
                    queue_id,
                    confirmation_id,
                    messages,
                };

                Ok(result)
            }
            ALL_MESSAGES_DELIVERED_CONFIRMATION => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = TcpContract::NewMessagesConfirmation {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }
            CREATE_TOPIC_IF_NOT_EXISTS => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;

                let result = TcpContract::CreateTopicIfNotExists { topic_id };

                Ok(result)
            }

            REJECT => {
                let message =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let result = TcpContract::Reject { message };
                Ok(result)
            }

            PACKET_VERSIONS => {
                let len = socket_reader.read_byte().await?;

                let mut packet_versions: HashMap<u8, i32> = HashMap::new();

                for _ in 0..len {
                    let p = socket_reader.read_byte().await?;
                    let v = socket_reader.read_i32().await?;
                    packet_versions.insert(p, v);
                }

                let result = TcpContract::PacketVersions { packet_versions };

                Ok(result)
            }

            ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = TcpContract::AllMessagesConfirmedAsFail {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }

            CONFIRM_SOME_MESSAGES_AS_OK => {
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let delivered =
                    super::common_deserializers::read_queue_with_intervals(socket_reader).await?;

                let result = TcpContract::ConfirmSomeMessagesAsOk {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                };

                Ok(result)
            }

            INTERMEDIARY_CONFIRM => {
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let delivered =
                    super::common_deserializers::read_queue_with_intervals(socket_reader).await?;

                let result = TcpContract::IntermediaryConfirm {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                };

                Ok(result)
            }

            _ => Err(ReadingTcpContractFail::InvalidPacketId(packet_no)),
        };

        return result;
    }

    pub fn serialize(self) -> Vec<u8> {
        match self {
            TcpContract::Ping {} => {
                let mut result: Vec<u8> = Vec::with_capacity(1);
                result.push(PING);
                result
            }
            TcpContract::Pong {} => {
                let mut result: Vec<u8> = Vec::with_capacity(1);
                result.push(PONG);
                result
            }
            TcpContract::Greeting {
                name,
                protocol_version,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(GREETING);
                serialize_pascal_string(&mut result, name.as_str());
                serialize_i32(&mut result, protocol_version);
                result
            }
            TcpContract::Publish {
                topic_id,
                request_id,
                persist_immediately,
                data_to_publish,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(PUBLISH);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_i64(&mut result, request_id);
                serialize_list_of_arrays(&mut result, &data_to_publish);
                serialize_bool(&mut result, persist_immediately);
                result
            }

            TcpContract::PublishResponse { request_id } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(PUBLISH_RESPONSE);
                serialize_i64(&mut result, request_id);
                result
            }
            TcpContract::Subscribe {
                topic_id,
                queue_id,
                queue_type,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(SUBSCRIBE);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_byte(&mut result, queue_type.into_u8());
                result
            }
            TcpContract::SubscribeResponse { topic_id, queue_id } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(SUBSCRIBE_RESPONSE);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                result
            }
            TcpContract::NewMessagesServerSide(payload) => payload,
            TcpContract::NewMessages {
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
                messages: _,
            } => {
                panic!(
                    "This packet is not used by server. Server uses optimized veriosn of the packet"
                );
            }
            TcpContract::NewMessagesConfirmation {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(ALL_MESSAGES_DELIVERED_CONFIRMATION);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);
                result
            }
            TcpContract::CreateTopicIfNotExists { topic_id } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(CREATE_TOPIC_IF_NOT_EXISTS);
                serialize_pascal_string(&mut result, topic_id.as_str());
                result
            }
            TcpContract::IntermediaryConfirm {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(INTERMEDIARY_CONFIRM);
                result.push(packet_version);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);

                super::common_serializers::serialize_queue_with_intervals(&mut result, &delivered);
                result
            }
            TcpContract::PacketVersions { packet_versions } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(PACKET_VERSIONS);

                let data_len = packet_versions.len() as u8;
                serialize_byte(&mut result, data_len);

                for kv in packet_versions {
                    serialize_byte(&mut result, kv.0);
                    serialize_i32(&mut result, kv.1);
                }
                result
            }
            TcpContract::Reject { message } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(REJECT);
                serialize_pascal_string(&mut result, message.as_str());
                result
            }
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);
                result
            }

            TcpContract::ConfirmSomeMessagesAsOk {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(CONFIRM_SOME_MESSAGES_AS_OK);
                result.push(packet_version);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);

                super::common_serializers::serialize_queue_with_intervals(&mut result, &delivered);
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use my_tcp_sockets::socket_reader::SocketReaderMock;

    use super::*;

    #[tokio::test]
    async fn test_ping_packet() {
        let tcp_packet = TcpContract::Ping;

        let mut socket_reader = SocketReaderMock::new();
        let attr = ConnectionAttributes::new();
        let serialized_data: Vec<u8> = tcp_packet.serialize();

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Ping => {}
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_pong_packet() {
        let tcp_packet = TcpContract::Pong;

        let mut socket_reader = SocketReaderMock::new();
        let attr = ConnectionAttributes::new();
        let serialized_data: Vec<u8> = tcp_packet.serialize();

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Pong => {}
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_greeting_packet() {
        let test_app_name = "testtttt";
        let test_protocol_version = 255;

        let tcp_packet = TcpContract::Greeting {
            name: test_app_name.to_string(),
            protocol_version: test_protocol_version,
        };

        let mut socket_reader = SocketReaderMock::new();

        let attr = ConnectionAttributes::new();

        let serialized_data: Vec<u8> = tcp_packet.serialize();

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Greeting {
                name,
                protocol_version,
            } => {
                assert_eq!(test_app_name, name);
                assert_eq!(test_protocol_version, protocol_version);
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_publish_packet() {
        let request_id_test = 1;
        let data_test = vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]];
        let topic_test = String::from("test-topic");
        let persist_test = true;

        let tcp_packet = TcpContract::Publish {
            data_to_publish: data_test,
            persist_immediately: persist_test,
            request_id: request_id_test,
            topic_id: topic_test,
        };

        let mut socket_reader = SocketReaderMock::new();

        let attr = ConnectionAttributes::new();

        let serialized_data: Vec<u8> = tcp_packet.serialize();

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Publish {
                data_to_publish,
                persist_immediately,
                request_id,
                topic_id,
            } => {
                assert_eq!(request_id_test, request_id);
                assert_eq!(String::from("test-topic"), topic_id);
                assert_eq!(persist_test, persist_immediately);

                let data_test = vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]];

                for index in 0..data_to_publish[0].len() {
                    assert_eq!(data_test[0][index], data_to_publish[0][index]);
                }
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_publish_response_packet() {
        let request_id_test = 1;

        let tcp_packet = TcpContract::PublishResponse {
            request_id: request_id_test,
        };

        let mut socket_reader = SocketReaderMock::new();

        let attr = ConnectionAttributes::new();

        let serialized_data: Vec<u8> = tcp_packet.serialize();

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::PublishResponse { request_id } => {
                assert_eq!(request_id_test, request_id);
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_packet() {
        let queue_id_test = String::from("queue");
        let topic_id_test = String::from("topic");
        let queue_type_test = TopicQueueType::PermanentWithSingleConnection;

        let tcp_packet = TcpContract::Subscribe {
            queue_id: queue_id_test,
            topic_id: topic_id_test,
            queue_type: queue_type_test,
        };

        let mut socket_reader = SocketReaderMock::new();

        let attr = ConnectionAttributes::new();

        let serialized_data: Vec<u8> = tcp_packet.serialize();

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Subscribe {
                queue_id,
                queue_type,
                topic_id,
            } => {
                let queue_id_test = String::from("queue");
                let topic_id_test = String::from("topic");

                assert_eq!(queue_id_test, queue_id);
                assert_eq!(topic_id_test, topic_id);
                match queue_type {
                    TopicQueueType::PermanentWithSingleConnection => {}
                    _ => {
                        panic!("Invalid Queue Type");
                    }
                };
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }
}
