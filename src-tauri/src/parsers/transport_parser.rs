use std::default;

use pnet::packet::{tcp::TcpPacket, udp::UdpPacket, Packet};

use crate::{
    parsers::parser::{LayerParser, ParseResult, ParseResultType},
    protocols::ProtocolId,
    reassembler::FragmentedPackets,
    Field, Layer, OsiLayer,
};

pub struct TcpParser;
pub struct UdpParser;

impl LayerParser for TcpParser {
    fn parse(bytes: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let tcp_packet = TcpPacket::new(&bytes)?;
        let flags = TcpFlag::from_u8(&tcp_packet.get_flags());

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
                    Field::new(
                        "Flags".to_string(),
                        flags
                            .iter()
                            .map(|flag| flag.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                    ),
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
            result_type: ParseResultType::Normal,
        })
    }
}

#[derive(Default)]
pub enum TcpFlag {
    FIN = 0,
    SYN = 1,
    RST = 2,
    PSH = 3,
    ACK = 4,
    URG = 5,
    #[default]
    Unknown,
}

impl TryFrom<u8> for TcpFlag {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TcpFlag::FIN),
            1 => Ok(TcpFlag::SYN),
            2 => Ok(TcpFlag::RST),
            3 => Ok(TcpFlag::PSH),
            4 => Ok(TcpFlag::ACK),
            5 => Ok(TcpFlag::URG),
            _ => Err(()),
        }
    }
}

impl ToString for TcpFlag {
    fn to_string(&self) -> String {
        match self {
            TcpFlag::FIN => "FIN".to_string(),
            TcpFlag::SYN => "SYN".to_string(),
            TcpFlag::RST => "RST".to_string(),
            TcpFlag::PSH => "PSH".to_string(),
            TcpFlag::ACK => "ACK".to_string(),
            TcpFlag::URG => "URG".to_string(),
            TcpFlag::Unknown => "Unknown".to_string(),
        }
    }
}

impl TcpFlag {
    pub fn from_u8(val: &u8) -> Vec<Self> {
        (0..6)
            .into_iter()
            .filter_map(|i| {
                if (1 << i) & val != 0 {
                    Some(TcpFlag::try_from(i).unwrap_or_default())
                } else {
                    None
                }
            })
            .collect()
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
            result_type: ParseResultType::Normal,
        })
    }
}
