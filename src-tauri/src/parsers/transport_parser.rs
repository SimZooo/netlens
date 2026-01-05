use std::collections::BTreeMap;

use pnet::packet::{
    ethernet::EtherTypes, ip::IpNextHeaderProtocols, tcp::TcpPacket, udp::UdpPacket,
};

use crate::{
    parsers::parser::{LayerParser, PacketContext},
    Field, FragmentedPackets, IpFragmentedPacket, Layer, OsiLayer,
};

pub struct TransportParser;

impl LayerParser for TransportParser {
    fn parse(
        _: &Vec<u8>,
        packet_context: &mut PacketContext,
        _: Option<&mut FragmentedPackets>,
    ) -> Option<Layer> {
        // TODO: Handle IPv6
        match packet_context.ethertype? {
            EtherTypes::Ipv4 => Self::handle_v4(packet_context),
            _ => {
                return None;
            }
        }
    }
}

impl TransportParser {
    fn handle_v4(packet_context: &mut PacketContext) -> Option<Layer> {
        let protocol = packet_context.ip_next_level_prot?;
        let payload = &packet_context.network_payload;
        match protocol {
            IpNextHeaderProtocols::Tcp => {
                let Some(tcp_packet) = TcpPacket::new(payload) else {
                    return None;
                };

                Some(Layer {
                    name: "TCP Packet".to_string(),
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
                })
            }
            IpNextHeaderProtocols::Udp => {
                let Some(udp_packet) = UdpPacket::new(payload) else {
                    return None;
                };

                Some(Layer {
                    name: "UDP Packet".to_string(),
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
                })
            }
            _ => Some(Layer {
                name: protocol.to_string(),
                osi_layer: OsiLayer::Transport,
                fields: vec![],
            }),
        }
    }
}
