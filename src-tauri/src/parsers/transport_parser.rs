use pnet::packet::{tcp::TcpPacket, udp::UdpPacket, Packet};

use crate::{
    parsers::parser::{LayerParser, ParseResult},
    protocols::ProtocolId,
    reassembler::FragmentedPackets,
    Field, Layer, OsiLayer,
};

pub struct TcpParser;
pub struct UdpParser;

impl LayerParser for TcpParser {
    fn parse(bytes: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let tcp_packet = TcpPacket::new(&bytes)?;

        Some(ParseResult {
            layer: Layer {
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
            },
            next: ProtocolId::None,
            remaining: tcp_packet.payload().to_vec(),
        })
    }
}

impl LayerParser for UdpParser {
    fn parse(input: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let udp_packet = UdpPacket::new(&input)?;

        Some(ParseResult {
            layer: Layer {
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
            },
            next: ProtocolId::None,
            remaining: udp_packet.payload().to_vec(),
        })
    }
}
