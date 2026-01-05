use std::collections::BTreeMap;

use pnet::packet::{ethernet::EtherType, ip::IpNextHeaderProtocol};

use crate::{FragmentedPackets, IpFragmentedPacket, Layer};

#[derive(Clone, Default)]
pub struct PacketContext {
    pub ethertype: Option<EtherType>,
    pub datalink_payload: Vec<u8>,
    pub network_payload: Vec<u8>,
    pub ip_next_level_prot: Option<IpNextHeaderProtocol>,
}

pub trait LayerParser {
    fn parse(
        data: &Vec<u8>,
        packet_context: &mut PacketContext,
        fragmented_packets: Option<&mut FragmentedPackets>,
    ) -> Option<Layer>;
}
