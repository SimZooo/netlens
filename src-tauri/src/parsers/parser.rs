use pnet::packet::ethernet::EtherType;

use crate::{protocols::ProtocolNames, FragmentedPackets, Layer};

#[derive(Clone, Default)]
pub struct PacketContext {
    pub src: String,
    pub dst: String,
    pub ethertype: Option<EtherType>,
    pub datalink_payload: Vec<u8>,
    pub network_payload: Vec<u8>,
    pub next_protocol: String,
}

pub trait LayerParser {
    fn parse(
        data: &Vec<u8>,
        packet_context: &mut PacketContext,
        fragmented_packets: Option<&mut FragmentedPackets>,
    ) -> Option<Vec<Layer>>;
}
