#![cfg(test)]

use super::{Batch, PacketError};
use packets::RawPacket;

pub struct PacketBatch {
    packets: Vec<RawPacket>,
}

impl PacketBatch {
    pub fn new(data: &[u8]) -> PacketBatch {
        PacketBatch {
            packets: vec![
                RawPacket::from_bytes(data).unwrap(),
                RawPacket::from_bytes(data).unwrap(),
                RawPacket::from_bytes(data).unwrap(),
            ],
        }
    }
}

impl Batch for PacketBatch {
    type Item = RawPacket;

    fn next(&mut self) -> Option<Result<Self::Item, PacketError>> {
        self.packets.pop().map(|p| Ok(p))
    }

    fn receive(&mut self) {
        // nop
    }
}
