use std::collections::{BTreeMap, HashMap};

use pnet::packet::{
    arp::ArpPacket,
    ethernet::{EtherType, EtherTypes, EthernetPacket},
    Packet,
};

use crate::{
    parsers::parser::{LayerParser, PacketContext},
    Field, FragmentedPackets, IpFragmentedPacket, Layer, OsiLayer,
};

pub struct EthernetParser {}
pub struct ArpParser {}

impl LayerParser for EthernetParser {
    fn parse(
        data: &Vec<u8>,
        packet_context: &mut PacketContext,
        _: Option<&mut FragmentedPackets>,
    ) -> Option<Vec<Layer>> {
        // Data link
        let Some(ethernet_packet) = EthernetPacket::new(data) else {
            return None;
        };

        packet_context.ethertype = Some(ethernet_packet.get_ethertype());
        packet_context.datalink_payload = ethernet_packet.payload().to_vec();
        packet_context.network_payload = vec![];
        packet_context.next_protocol = ethernet_packet.get_ethertype().to_string();
        if ethernet_packet.get_ethertype() == EtherTypes::Arp {
            println!("{}", packet_context.next_protocol);
        }
        packet_context.src = ethernet_packet.get_source().to_string();
        packet_context.dst = ethernet_packet.get_destination().to_string();

        Some(vec![Layer {
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
        }])
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
            Self::Reply => "Reply".to_string(),
            Self::Request => "Reply".to_string(),
            Self::Unknown => "Unknow".to_string(),
        }
    }
}

impl LayerParser for ArpParser {
    fn parse(
        _: &Vec<u8>,
        packet_context: &mut PacketContext,
        _: Option<&mut FragmentedPackets>,
    ) -> Option<Vec<Layer>> {
        let payload = packet_context.datalink_payload.clone();
        let Some(arp_packet) = ArpPacket::new(&payload) else {
            return None;
        };

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

        packet_context.network_payload = vec![];
        packet_context.next_protocol = "".to_string();
        packet_context.src = arp_packet.get_sender_proto_addr().to_string();
        packet_context.dst = arp_packet.get_target_hw_addr().to_string();

        Some(vec![Layer {
            protocol: "Arp".to_string(),
            name: "Arp Packet".to_string(),
            fields,
            osi_layer: OsiLayer::DataLink,
        }])
    }
}
