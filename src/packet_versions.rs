use std::collections::HashMap;

pub struct PacketVersions {
    versions: Vec<i32>,
}

impl PacketVersions {
    pub fn new() -> PacketVersions {
        PacketVersions {
            versions: vec![0i32; 256],
        }
    }

    pub fn get_packet_version(&self, packet_no: u8) -> i32 {
        unsafe {
            return self.versions.get_unchecked(packet_no as usize).clone();
        }
    }

    pub fn update(&mut self, data: &HashMap<u8, i32>) {
        for (i, v) in data {
            self.versions[*i as usize] = *v
        }
    }
}
