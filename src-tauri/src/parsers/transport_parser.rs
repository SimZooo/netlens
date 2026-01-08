use std::collections::BTreeMap;

use pnet::packet::{
    ethernet::EtherTypes,
    icmp::IcmpPacket,
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
    tcp::TcpPacket,
    udp::UdpPacket,
};

use crate::{
    parsers::parser::{LayerParser, PacketContext},
    Field, FragmentedPackets, Layer, OsiLayer,
};

pub struct TcpParser;
pub struct UdpParser;

impl LayerParser for TcpParser {
    fn parse(
        _: &Vec<u8>,
        packet_context: &mut PacketContext,
        _: Option<&mut FragmentedPackets>,
    ) -> Option<Vec<Layer>> {
        let payload = &packet_context.network_payload;
        let Some(tcp_packet) = TcpPacket::new(payload) else {
            return None;
        };
        packet_context.next_protocol = "".to_string();

        Some(vec![Layer {
            protocol: "Tcp".to_string(),
            name: "Tcp Packet".to_string(),
            osi_layer: OsiLayer::Transport,
            fields: vec![
                Field::new("Source".to_string(), tcp_packet.get_source().to_string()),
                Field::new(
                    "Destination".to_string(),
                    tcp_packet.get_destination().to_string(),
                ),
                Field::new("Flags".to_string(), tcp_packet.get_flags().to_string()),
                Field::new(
                    "Checksum".to_string(),
                    tcp_packet.get_checksum().to_string(),
                ),
                Field::new(
                    "Sequence number".to_string(),
                    tcp_packet.get_sequence().to_string(),
                ),
            ],
        }])
    }
}

impl LayerParser for UdpParser {
    fn parse(
        _: &Vec<u8>,
        packet_context: &mut PacketContext,
        _: Option<&mut FragmentedPackets>,
    ) -> Option<Vec<Layer>> {
        let payload = &packet_context.network_payload;
        let udp_packet = UdpPacket::new(payload)?;
        packet_context.next_protocol = "".to_string();

        Some(vec![Layer {
            protocol: "Udp".to_string(),
            name: "Udp Packet".to_string(),
            osi_layer: OsiLayer::Transport,
            fields: vec![
                Field::new("Source".to_string(), udp_packet.get_source().to_string()),
                Field::new(
                    "Destination".to_string(),
                    udp_packet.get_destination().to_string(),
                ),
                Field::new(
                    "Checksum".to_string(),
                    udp_packet.get_checksum().to_string(),
                ),
                Field::new("Length".to_string(), udp_packet.get_length().to_string()),
            ],
        }])
    }
}
