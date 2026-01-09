use std::net::IpAddr;

use pnet::packet::{
    ethernet::EtherTypes,
    icmp::{IcmpCode, IcmpPacket, IcmpType, IcmpTypes},
    icmpv6::Icmpv6Packet,
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
    ipv4::Ipv4Packet,
    ipv6::{ExtensionIterable, Ipv6Packet},
    Packet,
};

use crate::{
    parsers::parser::{LayerParser, ParseInput, ParseResult},
    protocols::ProtocolId,
    reassembler::{FragmentedKey, FragmentedPackets, IpFragmentedPacket},
    Field, Layer, OsiLayer,
};

pub struct Ipv4Parser;
pub struct Ipv6Parser;
pub struct IcmpParser;
pub struct Icmpv6Parser;

impl LayerParser for Ipv4Parser {
    fn parse(
        input: &[u8],
        fragmented_packets: Option<&mut FragmentedPackets>,
    ) -> Option<ParseResult> {
        let ip_packet = Ipv4Packet::new(&input)?;
        let fragmented =
            (ip_packet.get_flags() & 0b001) != 0 || ip_packet.get_fragment_offset() != 0;
        let mut next = ProtocolId::from_ip(ip_packet.get_next_level_protocol());

        let mut fields = vec![
            Field::new("Source".to_string(), ip_packet.get_source().to_string()),
            Field::new(
                "Destination".to_string(),
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
            next = ProtocolId::None;
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
                        false,
                        ip_packet.get_next_level_protocol(),
                        EtherTypes::Ipv4,
                        ip_packet.payload().to_vec(),
                        ip_packet.get_fragment_offset() as usize,
                        fields.clone(),
                    ),
                );
            }

            fields.insert(
                0,
                Field::new(
                    "Fragmented".to_string(),
                    format!("Offset: {}", ip_packet.get_fragment_offset().to_string()),
                ),
            );
        }

        Some(ParseResult {
            layer: Layer {
                protocol: ip_packet.get_next_level_protocol().to_string(),
                name: "Internet Protocol Version 4".to_string(),
                osi_layer: OsiLayer::Network,
                fields,
            },
            next,
            remaining: ip_packet.payload().to_vec(),
        })
    }
}

impl LayerParser for Ipv6Parser {
    fn parse(input: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let ip_packet = Ipv6Packet::new(&input)?;
        let mut fields = vec![
            Field::new("Source".to_string(), ip_packet.get_source().to_string()),
            Field::new(
                "Destination".to_string(),
                ip_packet.get_destination().to_string(),
            ),
            Field::new("Version".to_string(), ip_packet.get_version().to_string()),
            Field::new(
                "Hop Limit".to_string(),
                ip_packet.get_hop_limit().to_string(),
            ),
            Field::new(
                "Flow Label".to_string(),
                ip_packet.get_flow_label().to_string(),
            ),
            Field::new(
                "Traffic Class".to_string(),
                ip_packet.get_traffic_class().to_string(),
            ),
        ];

        let extensions = ExtensionIterable::new(ip_packet.payload());
        let mut top_level_protocol = None;

        for ext in extensions {
            match ext.get_next_header() {
                IpNextHeaderProtocols::Tcp
                | IpNextHeaderProtocols::Udp
                | IpNextHeaderProtocols::Icmpv6
                | IpNextHeaderProtocols::Icmp => {
                    break;
                }
                _ => fields.push(Field::new(
                    "Ipv6 Extension".to_string(),
                    ext.get_next_header().to_string(),
                )),
            }
            top_level_protocol = Some(ext.get_next_header());
        }

        Some(ParseResult {
            layer: Layer {
                protocol: "Ipv6".to_string(),
                name: "Internet Protocol Version 6".to_string(),
                osi_layer: OsiLayer::Network,
                fields,
            },
            next: ProtocolId::from_ip(top_level_protocol.unwrap_or(ip_packet.get_next_header())),
            remaining: ip_packet.payload().to_vec(),
        })
    }
}

impl LayerParser for IcmpParser {
    fn parse(input: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let icmp_packet = IcmpPacket::new(&input)?;
        println!("heyo");
        Some(ParseResult {
            layer: Layer {
                name: "Icmp Packet".to_string(),
                protocol: "Icmp".to_string(),
                osi_layer: OsiLayer::Network,
                fields: vec![
                    Field::new(
                        "ICMP Type".to_string(),
                        Self::icmp_type(icmp_packet.get_icmp_type()).to_string(),
                    ),
                    Field::new(
                        "ICMP Code".to_string(),
                        Self::icmp_code(icmp_packet.get_icmp_code()).to_string(),
                    ),
                    Field::new(
                        "Checksum".to_string(),
                        icmp_packet.get_checksum().to_string(),
                    ),
                ],
            },
            next: ProtocolId::None,
            remaining: icmp_packet.payload().to_vec(),
        })
    }
}

impl LayerParser for Icmpv6Parser {
    fn parse(input: &[u8], _: Option<&mut FragmentedPackets>) -> Option<ParseResult> {
        let icmp_packet = Icmpv6Packet::new(&input)?;
        Some(ParseResult {
            layer: Layer {
                name: "Icmpv6 Packet".to_string(),
                protocol: "Icmpv6".to_string(),
                osi_layer: OsiLayer::Network,
                fields: vec![
                    Field::new(
                        "ICMP Type".to_string(),
                        format!("{:?}", icmp_packet.get_icmpv6_type()),
                    ),
                    Field::new(
                        "ICMP Code".to_string(),
                        format!("{:?}", icmp_packet.get_icmpv6_code()),
                    ),
                    Field::new(
                        "Checksum".to_string(),
                        icmp_packet.get_checksum().to_string(),
                    ),
                ],
            },
            next: ProtocolId::None,
            remaining: icmp_packet.payload().to_vec(),
        })
    }
}

impl IcmpParser {
    fn icmp_type(icmp_type: IcmpType) -> &'static str {
        match icmp_type.0 {
            0 => "EchoReply",
            3 => "DestinationUnreachable",
            4 => "SourceQuench",
            5 => "RedirectMessage",
            8 => "EchoRequest",
            9 => "RouterAdvertisement",
            10 => "RouterSolicitation",
            11 => "TimeExceeded",
            12 => "ParameterProblem",
            13 => "Timestamp",
            14 => "TimestampReply",
            15 => "InformationRequest",
            16 => "InformationReply",
            17 => "AddressMaskRequest",
            18 => "AddressMaskReply",
            30 => "Traceroute",
            _ => "Unknown",
        }
    }

    fn icmp_code(icmp_code: IcmpCode) -> &'static str {
        match icmp_code.0 {
            0 => "DestinationNetworkUnreachable",
            1 => "DestinationHostUnreachable",
            2 => "DestinationProtocolUnreachable",
            3 => "DestinationPortUnreachable",
            4 => "FragmentationRequiredAndDFFlagSet",
            5 => "SourceRouteFailed",
            6 => "DestinationNetworkUnknown",
            7 => "DestinationHostUnknown",
            8 => "SourceHostIsolated",
            9 => "NetworkAdministrativelyProhibited",
            10 => "HostAdministrativelyProhibited",
            11 => "NetworkUnreachableForTOS",
            12 => "HostUnreachableForTOS",
            13 => "CommunicationAdministrativelyProhibited",
            14 => "HostPrecedenceViolation",
            15 => "PrecedenceCutoffInEffect",
            _ => "Unknown",
        }
    }
}
