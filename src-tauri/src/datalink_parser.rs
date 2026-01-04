use pnet::packet::{
    ethernet::{EtherType, EthernetPacket},
    Packet,
};

use crate::{
    parser::{LayerParser, PacketMetadata},
    Field, Layer, OsiLayer,
};

pub struct DatalinkMetadata {
    ether_type: EtherType,
}

pub struct DatalinkParser {}

impl LayerParser for DatalinkParser {
    fn parse(data: &[u8]) -> Option<(Layer, PacketMetadata)> {
        // Data link
        let Some(ethernet_packet) = EthernetPacket::new(data) else {
            return None;
        };

        let metadata = PacketMetadata {
            ethertype: Some(ethernet_packet.get_ethertype()),
            datalink_payload: Some(ethernet_packet.payload().to_vec()),
        };

        Some((
            Layer {
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
            },
            metadata,
        ))
    }
}
