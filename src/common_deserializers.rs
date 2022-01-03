use my_service_bus_shared::queue_with_intervals::QueueIndexRange;
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

pub async fn read_pascal_string<T: SocketReader>(
    reader: &mut T,
) -> Result<String, ReadingTcpContractFail> {
    let size = reader.read_byte().await? as usize;

    let mut result: Vec<u8> = Vec::with_capacity(size);
    unsafe { result.set_len(size) }

    reader.read_buf(&mut result).await?;

    Ok(String::from_utf8(result)?)
}

pub async fn read_queue_with_intervals<T: SocketReader>(
    reader: &mut T,
) -> Result<Vec<QueueIndexRange>, ReadingTcpContractFail> {
    let len = reader.read_i32().await?;

    let mut result: Vec<QueueIndexRange> = Vec::new();

    for _ in 0..len {
        let from_id = reader.read_i64().await?;
        let to_id = reader.read_i64().await?;

        result.push(QueueIndexRange { from_id, to_id });
    }

    Ok(result)
}
