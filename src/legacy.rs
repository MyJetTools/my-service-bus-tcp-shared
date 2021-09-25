use crate::PacketProtVer;

use super::{
    common_serializers::*, tcp_contracts::RequestId, ConnectionAttributes, ReadingTcpContractFail,
    TSocketReader,
};

pub async fn read_long<T: TSocketReader>(
    data_reader: &mut T,
    ver: &PacketProtVer,
) -> Result<i64, ReadingTcpContractFail> {
    if ver.protocol_version >= 2 {
        return data_reader.read_i64().await;
    }

    let result = data_reader.read_i32().await?;
    Ok(result as i64)
}

pub async fn read_long_with_connection_attr<T: TSocketReader>(
    data_reader: &mut T,
    attr: &ConnectionAttributes,
) -> Result<i64, ReadingTcpContractFail> {
    if attr.protocol_version >= 2 {
        return data_reader.read_i64().await;
    }

    let result = data_reader.read_i32().await?;
    Ok(result as i64)
}

pub fn serialize_long(data: &mut Vec<u8>, request_id: RequestId, attr: &ConnectionAttributes) {
    if attr.protocol_version < 2 {
        serialize_i32(data, request_id as i32);
    } else {
        serialize_i64(data, request_id);
    }
}
