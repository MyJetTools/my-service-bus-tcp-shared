use async_trait::async_trait;
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};

use crate::{ConnectionAttributes, TcpContract};

pub struct MySbTcpSerializer {
    attr: ConnectionAttributes,
}

impl MySbTcpSerializer {
    pub fn new(attr: ConnectionAttributes) -> Self {
        Self { attr }
    }
}

#[async_trait]
impl TcpSocketSerializer<TcpContract> for MySbTcpSerializer {
    fn serialize(&self, contract: TcpContract) -> Vec<u8> {
        contract.serialize()
    }
    fn get_ping_payload(&self) -> Vec<u8> {
        TcpContract::Ping.serialize()
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<TcpContract, ReadingTcpContractFail> {
        let result = TcpContract::deserialize(socket_reader, &self.attr).await?;
        Ok(result)
    }

    fn apply_packet(&mut self, contract: &TcpContract) {
        match contract {
            TcpContract::Greeting {
                name: _,
                protocol_version,
            } => {
                self.attr.protocol_version = *protocol_version;
            }
            TcpContract::PacketVersions { packet_versions } => {
                self.attr.versions.update(packet_versions);
            }
            _ => {}
        }
    }
}
