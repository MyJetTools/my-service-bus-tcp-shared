pub mod common_deserializers;
pub mod common_serializers;
pub mod deserializers;
pub mod legacy;
pub mod tcp_contract_to_string;
pub mod tcp_message_id;

mod connection_attrs;
mod error;
mod packet_versions;
mod socket_reader;
mod tcp_contracts;

pub use connection_attrs::ConnectionAttributes;
pub use error::ReadingTcpContractFail;
pub use packet_versions::PacketVersions;
pub use socket_reader::{SocketReader, TSocketReader};
pub use tcp_contracts::{PacketProtVer, TcpContract, TcpContractMessage};
