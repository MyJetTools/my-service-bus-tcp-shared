use super::PacketProtVer;

use super::common_serializers::*;

pub fn serialize_long(payload: &mut Vec<u8>, value: i64, ver: &PacketProtVer) {
    if ver.protocol_version < 2 {
        serialize_i32(payload, value as i32);
    } else {
        serialize_i64(payload, value);
    }
}
