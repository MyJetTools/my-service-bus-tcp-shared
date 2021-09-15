use super::{
    common_serializers::*, tcp_contracts::ConnectionAttributes, tcp_contracts::RequestId,
    MySbSocketError, TSocketReader,
};

pub async fn read_long<T: TSocketReader>(
    data_reader: &mut T,
    attr: &ConnectionAttributes,
) -> Result<i64, MySbSocketError> {
    if attr.protocol_version >= 2 {
        return data_reader.read_i64().await;
    }

    return match data_reader.read_i32().await {
        Ok(res) => Ok(res as i64),
        Err(err) => Err(err),
    };
}

pub fn serialize_long(data: &mut Vec<u8>, request_id: RequestId, attr: &ConnectionAttributes) {
    if attr.protocol_version < 2 {
        serialize_i32(data, request_id as i32);
    } else {
        serialize_i64(data, request_id);
    }
}