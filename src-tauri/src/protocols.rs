use crate::{
    parsers::{
        datalink_parser::ArpParser,
        network_parser::{IcmpParser, Icmpv6Parser, Ipv4Parser, Ipv6Parser},
        parser::{LayerParser, PacketContext},
        transport_parser::{TcpParser, UdpParser},
    },
    FragmentedPackets, Layer, OsiLayer,
};

#[derive(Clone, Default)]
pub struct Protocol {
    pub name: ProtocolNames,
    pub osi_layer: OsiLayer,
    pub parser: Option<
        fn(&Vec<u8>, &mut PacketContext, Option<&mut FragmentedPackets>) -> Option<Vec<Layer>>,
    >,
}

#[derive(Clone, Default, PartialEq)]
pub enum ProtocolNames {
    #[default]
    Ipv4,
    Ipv6,
    Tcp,
    Udp,
    Arp,
    Icmp,
    Icmpv6,
}

impl ToString for ProtocolNames {
    fn to_string(&self) -> String {
        match *self {
            Self::Ipv4 => "Ipv4".to_string(),
            Self::Ipv6 => "Ipv6".to_string(),
            Self::Tcp => "Tcp".to_string(),
            Self::Udp => "Udp".to_string(),
            Self::Arp => "Arp".to_string(),
            Self::Icmp => "Icmp".to_string(),
            Self::Icmpv6 => "Icmpv6".to_string(),
        }
    }
}

pub static PROTOCOLS: &[Protocol] = &[
    Protocol {
        name: ProtocolNames::Ipv4,
        osi_layer: OsiLayer::Network,
        parser: Some(Ipv4Parser::parse),
    },
    Protocol {
        name: ProtocolNames::Ipv6,
        osi_layer: OsiLayer::Network,
        parser: Some(Ipv6Parser::parse),
    },
    Protocol {
        name: ProtocolNames::Tcp,
        osi_layer: OsiLayer::Transport,
        parser: Some(TcpParser::parse),
    },
    Protocol {
        name: ProtocolNames::Udp,
        osi_layer: OsiLayer::Transport,
        parser: Some(UdpParser::parse),
    },
    Protocol {
        name: ProtocolNames::Icmp,
        osi_layer: OsiLayer::Network,
        parser: Some(IcmpParser::parse),
    },
    Protocol {
        name: ProtocolNames::Icmpv6,
        osi_layer: OsiLayer::Network,
        parser: Some(Icmpv6Parser::parse),
    },
    Protocol {
        name: ProtocolNames::Arp,
        osi_layer: OsiLayer::Network,
        parser: Some(ArpParser::parse),
    },
];
