use pnet::packet::ethernet::EtherType;

use crate::Layer;

pub struct PacketMetadata {
    pub ethertype: Option<EtherType>,
    pub datalink_payload: Option<Vec<u8>>,
}

pub trait LayerParser {
    fn parse(data: &[u8]) -> Option<(Layer, PacketMetadata)>;
}
