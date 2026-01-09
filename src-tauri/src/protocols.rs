use pnet::packet::{
    ethernet::{EtherType, EtherTypes},
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
};

use crate::{
    parsers::{
        datalink_parser::{ArpParser, EthernetParser},
        network_parser::{IcmpParser, Icmpv6Parser, Ipv4Parser, Ipv6Parser},
        parser::{LayerParser, ParseInput, ParseResult},
        transport_parser::{TcpParser, UdpParser},
    },
    reassembler::FragmentedPackets,
    Layer, OsiLayer,
};

#[derive(Clone, Default)]
pub struct Protocol {
    pub name: ProtocolId,
    pub osi_layer: OsiLayer,
    pub parser: Option<fn(&[u8], Option<&mut FragmentedPackets>) -> Option<ParseResult>>,
}

#[derive(Clone, Default, PartialEq)]
pub enum ProtocolId {
    Ethernet,
    Arp,
    Ipv4,
    Ipv6,
    Tcp,
    Udp,
    Icmp,
    Icmpv6,
    #[default]
    None,
}

impl ProtocolId {
    pub fn from_ethertype(ether_type: EtherType) -> Self {
        match ether_type {
            EtherTypes::Arp => ProtocolId::Arp,
            EtherTypes::Ipv4 => ProtocolId::Ipv4,
            EtherTypes::Ipv6 => ProtocolId::Ipv6,
            _ => ProtocolId::None,
        }
    }
    pub fn from_ip(next: IpNextHeaderProtocol) -> Self {
        match next {
            IpNextHeaderProtocols::Tcp => ProtocolId::Tcp,
            IpNextHeaderProtocols::Udp => ProtocolId::Udp,
            IpNextHeaderProtocols::Icmp => ProtocolId::Icmp,
            IpNextHeaderProtocols::Icmpv6 => ProtocolId::Icmpv6,
            _ => ProtocolId::None,
        }
    }
}

impl ToString for ProtocolId {
    fn to_string(&self) -> String {
        match *self {
            Self::Ethernet => "Ethernet".to_string(),
            Self::Ipv4 => "Ipv4".to_string(),
            Self::Ipv6 => "Ipv6".to_string(),
            Self::Tcp => "Tcp".to_string(),
            Self::Udp => "Udp".to_string(),
            Self::Arp => "Arp".to_string(),
            Self::Icmp => "Icmp".to_string(),
            Self::Icmpv6 => "Icmpv6".to_string(),
            Self::None => "".to_string(),
        }
    }
}

pub static PROTOCOLS: &[Protocol] = &[
    Protocol {
        name: ProtocolId::Ethernet,
        osi_layer: OsiLayer::DataLink,
        parser: Some(EthernetParser::parse),
    },
    Protocol {
        name: ProtocolId::Ipv4,
        osi_layer: OsiLayer::Network,
        parser: Some(Ipv4Parser::parse),
    },
    Protocol {
        name: ProtocolId::Ipv6,
        osi_layer: OsiLayer::Network,
        parser: Some(Ipv6Parser::parse),
    },
    Protocol {
        name: ProtocolId::Tcp,
        osi_layer: OsiLayer::Transport,
        parser: Some(TcpParser::parse),
    },
    Protocol {
        name: ProtocolId::Udp,
        osi_layer: OsiLayer::Transport,
        parser: Some(UdpParser::parse),
    },
    Protocol {
        name: ProtocolId::Icmp,
        osi_layer: OsiLayer::Network,
        parser: Some(IcmpParser::parse),
    },
    Protocol {
        name: ProtocolId::Icmpv6,
        osi_layer: OsiLayer::Network,
        parser: Some(Icmpv6Parser::parse),
    },
    Protocol {
        name: ProtocolId::Arp,
        osi_layer: OsiLayer::DataLink,
        parser: Some(ArpParser::parse),
    },
];
