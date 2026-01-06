use std::{collections::BTreeMap, net::IpAddr};

use pnet::packet::{ipv4::Ipv4Packet, Packet};

use crate::{
    parsers::parser::{LayerParser, PacketContext},
    reassembler::FragmentedKey,
    Field, FragmentedPackets, IpFragmentedPacket, Layer, OsiLayer,
};

pub struct Ipv4NetworkParser {}
pub struct Ipv6NetworkParser {}

impl LayerParser for Ipv4NetworkParser {
    fn parse(
        _: &Vec<u8>,
        packet_context: &mut PacketContext,
        fragmented_packets: Option<&mut FragmentedPackets>,
    ) -> Option<Layer> {
        let Some(ip_packet) = Ipv4Packet::new(&packet_context.datalink_payload[..]) else {
            return None;
        };
        let fragmented =
            (ip_packet.get_flags() & 0b001) != 0 || ip_packet.get_fragment_offset() != 0;

        let mut fields = vec![
            Field::new("Source IP".to_string(), ip_packet.get_source().to_string()),
            Field::new(
                "Destination IP".to_string(),
                ip_packet.get_destination().to_string(),
            ),
            Field::new("TTL".to_string(), ip_packet.get_ttl().to_string()),
            Field::new("Flags".to_string(), ip_packet.get_flags().to_string()),
            Field::new(
                "Identification".to_string(),
                ip_packet.get_identification().to_string(),
            ),
            Field::new("Version".to_string(), ip_packet.get_version().to_string()),
        ];

        let key = FragmentedKey {
            src_ip: IpAddr::V4(ip_packet.get_source()),
            dst_ip: IpAddr::V4(ip_packet.get_destination()),
            protocol: ip_packet.get_next_level_protocol(),
            identification: ip_packet.get_identification(),
        };

        if fragmented {
            fields.push(Field::new(
                "Fragmented".to_string(),
                format!("Offset: {}", ip_packet.get_fragment_offset().to_string()),
            ));

            let Some(fragmented_packets) = fragmented_packets else {
                println!("Fragmented Packets is None in NetworkParser, must be Some");
                return None;
            };

            if let Some(frag) = fragmented_packets.get_mut(&key) {
                println!("Existing fragmented packet");
                frag.fragments.insert(
                    ip_packet.get_fragment_offset() as usize,
                    ip_packet.payload().to_vec(),
                );
                if ip_packet.get_flags() & 0b001 == 0 {
                    // Check if all fragments exist
                    let mut expected = 0;
                    let mut done = true;
                    for f in frag.fragments.iter() {
                        println!("{} {}", f.0, expected);
                        if *f.0 != expected {
                            done = false;
                            break;
                        }
                        expected += 1;
                    }
                    frag.done = done;
                }
            } else {
                fragmented_packets.insert(
                    key,
                    IpFragmentedPacket::new_first(
                        IpAddr::V4(ip_packet.get_source()),
                        IpAddr::V4(ip_packet.get_destination()),
                        false,
                        ip_packet.get_next_level_protocol(),
                        packet_context.ethertype?,
                        ip_packet.payload().to_vec(),
                        ip_packet.get_fragment_offset() as usize,
                    ),
                );
            }
        }

        packet_context.network_payload = ip_packet.payload().to_vec();
        packet_context.ip_next_level_prot = Some(ip_packet.get_next_level_protocol());
        // Overwrite MAC address
        packet_context.src = ip_packet.get_source().to_string();
        packet_context.dst = ip_packet.get_destination().to_string();

        Some(Layer {
            protocol: ip_packet.get_next_level_protocol().to_string(),
            name: "Internet Protocol Version 4".to_string(),
            osi_layer: OsiLayer::Network,
            fields,
        })
    }
}
