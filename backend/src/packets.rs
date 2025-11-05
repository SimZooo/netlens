use std::{net::{IpAddr, Ipv4Addr}, time::Duration, vec};

use actix_web::{HttpRequest, HttpResponse, Responder, web};
use pnet::{datalink::{self, NetworkInterface}, ipnetwork::Ipv4Network, packet::{ethernet::{EtherTypes, EthernetPacket}, ipv4::Ipv4Packet, ipv6::Ipv6Packet, tcp::TcpPacket}, util::MacAddr};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::Packet;
use pnet::packet::FromPacket;
use serde::Serialize;
use tokio::{sync::broadcast, time::sleep};

use crate::arp_scan;

#[derive(Serialize)]
pub struct EthPkt {
    pub src: String,
    pub dst: String,
    pub packet_payload: Vec<u8>,
}

#[derive(Serialize)]
pub struct IpPkt {
    pub src: IpAddr,
    pub dst: IpAddr,
    pub packet_payload: Vec<u8>,
}

#[derive(Serialize)]
pub struct TcpPkt {
    pub src: u16,
    pub dst: u16,
    pub packet_payload: Vec<u8>,
}

#[derive(Default, Serialize)]
pub struct PacketDefinition {
    pub ethernet_packet: Option<EthPkt>,
    pub eth_packet: Option<EthPkt>,
    pub ip_packet: Option<IpPkt>,
    pub tcp_packet: Option<TcpPkt>,
}

pub async fn get_interfaces() -> impl Responder {
    let interfaces = datalink::interfaces();
    let names = interfaces.iter().map(|interface| (interface.name.clone(), interface.index)).collect::<Vec<(String, u32)>>();
    web::Json(names)
}

pub fn get_interface(index: u32) -> Option<NetworkInterface> {
    return datalink::interfaces().iter().find(|interface| interface.index == index).cloned();
}

pub async fn start_capture(tx: broadcast::Sender<Vec<u8>>) {
    let interfaces = datalink::interfaces();
    let interface = interfaces.get(0);
    println!("Started packet capture: {:?}!", interface);

    if let Some(interface) = interface {
        println!("Interface received! {:?}", interface.name);
        let (_, mut     rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unhandled interface"),
            Err(e) => panic!("An error ocurred when creating a datalink channel: {}", e)
        };
        tokio::spawn(async move {
            loop {
                match rx.next() {
                    Ok(packet) => {
                        let _ = tx.send(packet.to_vec());
                    }
                    Err(e) => eprintln!("Error getting packet: {e}")
                }
            }
        });
    }
}

pub async fn capture_interface_packets(req: HttpRequest, body: web::Payload, index: web::Path<u32>, tx: web::Data<broadcast::Sender<Vec<u8>>>) -> impl Responder {
    println!("Received WS connection!");
    //let interface = get_interface(index.into_inner());
    let (res, mut session, _) = actix_ws::handle(&req, body).unwrap();
    let mut rx = tx.subscribe();

    tokio::spawn(async move {
        while let Ok(packet) = rx.recv().await {
            if let Some(ethernet_packet) = EthernetPacket::new(&packet) {
                let packet_def = &mut PacketDefinition::default();

                packet_def.eth_packet = Some(EthPkt {
                    src: ethernet_packet.get_destination().to_string(),
                    dst: ethernet_packet.get_source().to_string(),
                    packet_payload: ethernet_packet.payload().to_vec(),
                });

                handle_ethertype(&ethernet_packet, packet_def);
                let packet_json = serde_json::to_string(&packet_def);

                if let Ok(text) = packet_json {
                    let _ = session.text(text).await;
                }
            }
        }
    });
    res
}

fn handle_ethertype(ethernet_packet: &EthernetPacket, packet_def: &mut PacketDefinition) {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ip_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                let ip_src = ip_packet.get_source();
                let ip_dst = ip_packet.get_destination();

                packet_def.ip_packet = Some(IpPkt {
                    src: IpAddr::V4(ip_src),
                    dst: IpAddr::V4(ip_dst),
                    packet_payload: ip_packet.payload().to_vec(),
                });

                if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
                    let port_src = tcp_packet.get_source();
                    let port_dst = tcp_packet.get_destination();

                    packet_def.tcp_packet = Some(TcpPkt {
                        src: port_src,
                        dst: port_dst,
                        packet_payload: tcp_packet.payload().to_vec(),
                    });
                }
            }
        },
        EtherTypes::Ipv6 => {
            if let Some(ip_packet) = Ipv6Packet::new(ethernet_packet.payload()) {
                let ip_src = ip_packet.get_source();
                let ip_dst = ip_packet.get_destination();

                packet_def.ip_packet = Some(IpPkt {
                    src: IpAddr::V6(ip_src),
                    dst: IpAddr::V6(ip_dst),
                    packet_payload: ip_packet.payload().to_vec(),
                });

                if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
                    let port_src = tcp_packet.get_source();
                    let port_dst = tcp_packet.get_destination();

                    packet_def.tcp_packet = Some(TcpPkt {
                        src: port_src,
                        dst: port_dst,
                        packet_payload: tcp_packet.payload().to_vec(),
                    });
                }
            }
        }
        _ => {}
    }
}

pub fn get_network_addr(interface: &NetworkInterface) -> Ipv4Network {
    let interface_ip = arp_scan::get_interface_ip(&interface);
    let ip_str = interface_ip.to_string();
    let network_parts = ip_str.split('.').map(|p| String::from(p)).collect::<Vec<String>>();
    let network_addr: String = format!("{}.0/24", network_parts[0..3].join("."));
    network_addr.parse().expect("Failed to get network address of interface")
}