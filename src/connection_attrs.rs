use super::PacketVersions;

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
}
