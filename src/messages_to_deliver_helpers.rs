use my_service_bus_shared::{messages_bucket::MessagesBucket, MySbMessage, MySbMessageContent};

use crate::common_serializers::{serialize_byte_array, serialize_i32};

pub async fn serialize_messages(
    result: &mut Vec<u8>,
    packet_version: i32,
    messages_to_deliver: &MessagesBucket,
) {
    let messages_count = messages_to_deliver.messages_count() as i32;

    serialize_i32(result, messages_count);

    let page_read_access = messages_to_deliver.page.data.read().await;

    for msg_id in &messages_to_deliver.ids {
        let message_info = messages_to_deliver.messages.get(&msg_id);

        if message_info.is_none() {
            println!("Somehow we did nof found message_info");
            continue;
        }

        let message_info = message_info.unwrap();

        let found_message = page_read_access.messages.get(&msg_id);

        if let Some(my_sb_message) = found_message {
            if let MySbMessage::Loaded(msg) = my_sb_message {
                serialize_message(result, msg, message_info.attempt_no, packet_version);
            } else {
                println!("Message is not loaded. Reason {:?}", my_sb_message)
            }
        } else {
            println!("Message not found to pack {:?}", msg_id)
        }
    }
}

pub fn serialize_message(
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
