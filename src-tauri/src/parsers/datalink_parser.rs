use pnet::packet::{arp::ArpPacket, ethernet::EthernetPacket, Packet};

use crate::{
    parsers::parser::{LayerParser, ParseInput, ParseResult},
    protocols::ProtocolId,
    reassembler::FragmentedPackets,
    Field, Layer, OsiLayer,
};

pub struct EthernetParser {}
pub struct ArpParser {}

impl LayerParser for EthernetParser {
    fn parse(input: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        // Data link
        let ethernet_packet = EthernetPacket::new(&input)?;

        Some(ParseResult {
            layer: Layer {
                protocol: "Ethernet".to_string(),
                name: "Ethernet Frame".to_string(),
                osi_layer: OsiLayer::DataLink,
                fields: vec![
                    Field::new(
                        "Source".to_string(),
                        format!("{:02x?}", ethernet_packet.get_source()),
                    ),
                    Field::new(
                        "Destination".to_string(),
                        format!("{:02x?}", ethernet_packet.get_destination()),
                    ),
                    Field::new(
                        "EtherType".to_string(),
                        format!("{}", ethernet_packet.get_ethertype().to_string()),
                    ),
                ],
            },
            next: ProtocolId::from_ethertype(ethernet_packet.get_ethertype()),
            remaining: ethernet_packet.payload().to_vec(),
        })
    }
}

pub enum ArpOperation {
    Request,
    Reply,
    Unknown,
}

impl ToString for ArpOperation {
    fn to_string(&self) -> String {
        match *self {
            Self::Request => "Request".to_string(),
            Self::Reply => "Reply".to_string(),
            Self::Unknown => "Unknow".to_string(),
        }
    }
}

impl LayerParser for ArpParser {
    fn parse(input: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let arp_packet = ArpPacket::new(&input)?;

        let mut fields = vec![];

        let arp_operation = match arp_packet.get_operation().0 {
            1 => {
                fields.push(Field::new(
                    "Who has".to_string(),
                    format!(
                        "{}? Send to {}({})",
                        arp_packet.get_target_proto_addr().to_string(),
                        arp_packet.get_sender_hw_addr().to_string(),
                        arp_packet.get_sender_proto_addr().to_string()
                    ),
                ));
                ArpOperation::Request
            }
            2 => {
                fields.push(Field::new(
                    "I have".to_string(),
                    format!(
                        "{}! My address is {}",
                        arp_packet.get_target_proto_addr().to_string(),
                        arp_packet.get_target_hw_addr().to_string()
                    ),
                ));
                ArpOperation::Reply
            }
            _ => ArpOperation::Unknown,
        };

        fields.push(Field::new(
            "Operation".to_string(),
            arp_operation.to_string(),
        ));
        fields.push(Field::new(
            "Sender  Addr".to_string(),
            arp_packet.get_sender_hw_addr().to_string(),
        ));
        fields.push(Field::new(
            "Sender Proto Addr".to_string(),
            arp_packet.get_sender_proto_addr().to_string(),
        ));
        fields.push(Field::new(
            "Target Proto Addr".to_string(),
            arp_packet.get_target_proto_addr().to_string(),
        ));
        fields.push(Field::new(
            "Proto Type".to_string(),
            arp_packet.get_protocol_type().to_string(),
        ));

        Some(ParseResult {
            layer: Layer {
                protocol: "Arp".to_string(),
                name: "Arp Packet".to_string(),
                fields,
                osi_layer: OsiLayer::DataLink,
            },
            next: ProtocolId::None,
            remaining: arp_packet.payload().to_vec(),
        })
    }
}
