use super::PacketVersions;

#[derive(Debug, Clone)]
pub struct PacketProtVer {
    pub packet_version: i32,
    pub protocol_version: i32,
}

#[derive(Clone)]
pub struct ConnectionAttributes {
    pub versions: PacketVersions,
    pub protocol_version: i32,
}

impl ConnectionAttributes {
    pub fn new() -> Self {
        Self {
            versions: PacketVersions::new(),
            protocol_version: 0,
        }
    }

    pub fn get(&self, packet_no: u8) -> PacketProtVer {
        PacketProtVer {
            protocol_version: self.protocol_version,
            packet_version: self.versions.get_packet_version(packet_no),
        }
    }
}
