use std::{collections::HashMap, str};

use my_service_bus_shared::queue_with_intervals::QueueIndexRange;

use crate::{tcp_contracts::MessageToPublishTcpContract, PacketProtVer};

pub fn serialize_byte(data: &mut Vec<u8>, v: u8) {
    data.push(v);
}

pub fn serialize_bool(data: &mut Vec<u8>, v: bool) {
    if v {
        data.push(1);
    } else {
        data.push(0);
    }
}

pub fn serialize_i32(data: &mut Vec<u8>, v: i32) {
    data.extend(&v.to_le_bytes());
}

pub fn serialize_i64(data: &mut Vec<u8>, v: i64) {
    data.extend(&v.to_le_bytes());
}

pub fn serialize_pascal_string(data: &mut Vec<u8>, str: &str) {
    let str_len = str.len() as u8;
    data.push(str_len);
    data.extend(str.as_bytes());
}

pub fn serialize_list_of_arrays(data: &mut Vec<u8>, v: &Vec<Vec<u8>>) {
    let array_len = v.len() as i32;
    serialize_i32(data, array_len);

    for arr in v {
        serialize_byte_array(data, arr);
    }
}

pub fn serialize_messages_v2(data: &mut Vec<u8>, v: &Vec<MessageToPublishTcpContract>) {
    let array_len = v.len() as i32;
    serialize_i32(data, array_len);

    for arr in v {
        serialize_byte_array(data, &arr.content);
    }
}

pub fn serialize_messages_v3(data: &mut Vec<u8>, v: &Vec<MessageToPublishTcpContract>) {
    let array_len = v.len() as i32;
    serialize_i32(data, array_len);

    for item in v {
        serialize_message_headers(data, item.headers.as_ref());
        serialize_byte_array(data, &item.content);
    }
}

pub fn serialize_message_headers(data: &mut Vec<u8>, headers: Option<&HashMap<String, String>>) {
    match headers {
        Some(headers) => {
            let mut headers_count = headers.len();

            if headers_count > 255 {
                headers_count = 255;
            }

            data.push(headers_count as u8);

            let mut i = 0;

            for (key, value) in headers {
                if i == 255 {
                    break;
                }

                serialize_pascal_string(data, key);
                serialize_pascal_string(data, value);

                i += 1;
            }
        }
        None => {
            data.push(0);
        }
    }
}

pub fn serialize_byte_array(data: &mut Vec<u8>, v: &[u8]) {
    let array_len = v.len() as i32;
    serialize_i32(data, array_len);
    data.extend(v);
}

pub fn serialize_queue_with_intervals(payload: &mut Vec<u8>, value: &Vec<QueueIndexRange>) {
    serialize_i32(payload, value.len() as i32);

    for itm in value {
        serialize_i64(payload, itm.from_id);
        serialize_i64(payload, itm.to_id);
    }
}

pub fn serialize_long(payload: &mut Vec<u8>, value: i64, ver: &PacketProtVer) {
    if ver.protocol_version < 2 {
        serialize_i32(payload, value as i32);
    } else {
        serialize_i64(payload, value);
    }
}
