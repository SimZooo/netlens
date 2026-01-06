use std::collections::{BTreeMap, HashMap};

use pnet::packet::{
    ethernet::{EtherType, EthernetPacket},
    Packet,
};

use crate::{
    parsers::parser::{LayerParser, PacketContext},
    Field, FragmentedPackets, IpFragmentedPacket, Layer, OsiLayer,
};

pub struct DatalinkMetadata {
    ether_type: EtherType,
}

pub struct DatalinkParser {}

impl LayerParser for DatalinkParser {
    fn parse(
        data: &Vec<u8>,
        packet_context: &mut PacketContext,
        _: Option<&mut FragmentedPackets>,
    ) -> Option<Layer> {
        // Data link
        let Some(ethernet_packet) = EthernetPacket::new(data) else {
            return None;
        };

        packet_context.ethertype = Some(ethernet_packet.get_ethertype());
        packet_context.datalink_payload = ethernet_packet.payload().to_vec();
        packet_context.network_payload = vec![];
        packet_context.ip_next_level_prot = None;
        packet_context.src = ethernet_packet.get_source().to_string();
        packet_context.dst = ethernet_packet.get_destination().to_string();

        Some(Layer {
            protocol: "Ethernet".to_string(),
            name: "Ethernet Frame".to_string(),
            osi_layer: OsiLayer::DataLink,
            fields: vec![
                Field::new(
                    "Source MAC".to_string(),
                    format!("{:02x?}", ethernet_packet.get_source()),
                ),
                Field::new(
                    "Destination MAC".to_string(),
                    format!("{:02x?}", ethernet_packet.get_destination()),
                ),
                Field::new(
                    "EtherType".to_string(),
                    format!("{}", ethernet_packet.get_ethertype().to_string()),
                ),
            ],
        })
    }
}
