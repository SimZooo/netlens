use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use std::thread;
use std::time::SystemTime;

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::icmp::{IcmpPacket, IcmpType, IcmpTypes};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    tcp::TcpPacket,
    udp::UdpPacket,
    Packet,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri::{Manager, State};

use crate::datalink_parser::DatalinkParser;
use crate::parser::LayerParser;

pub struct AppState {
    interface: Option<String>,
    listen: bool,
}

pub mod datalink_parser;
pub mod parser;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum OsiLayer {
    Physical,
    DataLink,
    Network,
    Transport,
    Session,
    Presentation,
    Application,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub value: String,
}

impl Field {
    pub fn new(name: String, value: String) -> Field {
        Field { name, value }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Layer {
    pub name: String,
    pub osi_layer: OsiLayer,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkPacket {
    pub id: String,
    pub timestamp: SystemTime,
    pub raw: Vec<u8>,
    pub layers: Vec<Layer>,
}

enum IpPacket<'a> {
    V4(Ipv4Packet<'a>),
    V6(Ipv6Packet<'a>),
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Arc::new(Mutex::new(AppState {
                interface: Some("\\Device\\NPF_{E9DBF177-F7A9-4288-900E-D19D1E210287}".to_string()),
                listen: false,
            })));
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            start_listening(
                app.state::<Arc<Mutex<AppState>>>(),
                app.app_handle().clone(),
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![set_listen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn set_listen(val: bool, state: State<'_, Arc<Mutex<AppState>>>) {
    let Ok(mut state) = state.lock() else {
        return;
    };

    println!("Set listen to {val}");

    state.listen = val;
}

fn start_listening(state: State<'_, Arc<Mutex<AppState>>>, app: AppHandle) {
    let state_clone = state.inner().clone();
    thread::spawn(move || {
        let mut fragmented_packets: HashMap<u16, Vec<Vec<u8>>> = HashMap::new();

        let interface_name = {
            let state = state_clone.lock().unwrap();
            state.interface.clone().unwrap()
        };
        let interfaces = datalink::interfaces();
        let interface = interfaces
            .iter()
            .find(|iface| iface.name == interface_name.clone())
            .expect("Invalid interface name");

        let (_, mut rx) = match datalink::channel(interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unhandled physical layer type"),
            Err(e) => panic!("Error creating interface channel: {e}"),
        };
        loop {
            let listen = {
                let Ok(state) = state_clone.lock() else {
                    return;
                };
                state.listen
            };

            if !listen {
                continue;
            }
            match rx.next() {
                Ok(packet) => {
                    let network_packet = handle_packet(packet, &mut fragmented_packets);
                    if let Some(p) = network_packet {
                        let _ = app.emit("packet_received", p);
                    }
                }
                Err(e) => eprintln!("{e}"),
            }
        }
    });
}

pub fn handle_packet(
    packet: &[u8],
    fragmented_packets: &mut HashMap<u16, Vec<Vec<u8>>>,
) -> Option<NetworkPacket> {
    let (datalink_layer, ethernet_metadata) = DatalinkParser::parse(packet).unwrap();
    let ether_type = ethernet_metadata.ethertype;
    let (network_layer, network_packet) = match ether_type {
        EtherTypes::Ipv4 => {
            let Some(ip_packet) = Ipv4Packet::new(ethernet_packet.payload()) else {
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
            ];
            if fragmented {
                fields.push(Field::new(
                    "Fragmented".to_string(),
                    format!("Offset: {}", ip_packet.get_fragment_offset().to_string()),
                ));

                if let Some(frags) = fragmented_packets.get_mut(&ip_packet.get_identification()) {
                    println!("Existing fragmented packet");
                    frags.push(ip_packet.payload().to_vec());
                    if ip_packet.get_flags() & 0b100 == 0 {
                        let payload: Vec<u8> = frags.iter().flatten().cloned().collect();
                        println!(
                            "Final packet of fragmented packet received: {:?} with protocol: {}",
                            payload,
                            ip_packet.get_next_level_protocol().to_string()
                        );
                    }
                } else {
                    fragmented_packets.insert(
                        ip_packet.get_identification(),
                        vec![ip_packet.payload().to_vec()],
                    );
                }
            }
            (
                Layer {
                    name: "Ipv4 Packet".to_string(),
                    osi_layer: OsiLayer::Network,
                    fields,
                },
                IpPacket::V4(ip_packet),
            )
        }
        /*
        EtherTypes::Ipv6 => {
            let Some(ip_packet) = Ipv6Packet::new(ethernet_packet.payload()) else {
                return None;
            };
            (
                Layer {
                    name: "Ipv6 Packet".to_string(),
                    osi_layer: OsiLayer::Network,
                    fields: vec![],
                },
                IpPacket::V6(ip_packet),
            )
        }
         */
        _ => return None,
    };

    let Some(transport_layer) = (match network_packet {
        IpPacket::V4(packet) => handle_v4(packet),
        IpPacket::V6(packet) => handle_v6(packet),
    }) else {
        return None;
    };

    Some(NetworkPacket {
        id: uuid::Uuid::new_v4().to_string(),
        layers: vec![datalink_layer, network_layer, transport_layer],
        raw: packet.to_vec(),
        timestamp: SystemTime::now(),
    })
}

fn handle_v4(network_packet: Ipv4Packet) -> Option<Layer> {
    let protocol = network_packet.get_next_level_protocol();
    match protocol {
        IpNextHeaderProtocols::Tcp => {
            let Some(tcp_packet) = TcpPacket::new(network_packet.payload()) else {
                return None;
            };

            Some(Layer {
                name: "TCP Packet".to_string(),
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
            })
        }
        IpNextHeaderProtocols::Udp => {
            let Some(udp_packet) = UdpPacket::new(network_packet.payload()) else {
                return None;
            };

            Some(Layer {
                name: "UDP Packet".to_string(),
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
            })
        }
        _ => Some(Layer {
            name: protocol.to_string(),
            osi_layer: OsiLayer::Transport,
            fields: vec![],
        }),
    }
}

fn handle_v6(network_packet: Ipv6Packet) -> Option<Layer> {
    let header = network_packet.get_next_header();
    let payload = network_packet.payload();
    let fields = vec![];
    if let Some(transport_protocol) = next_transport_header(payload) {
        return Some(Layer {
            name: transport_protocol.1.to_string(),
            osi_layer: OsiLayer::Transport,
            fields,
        });
    }

    return None;
}

fn next_transport_header<'a>(packet: &'a [u8]) -> Option<(&'a [u8], IpNextHeaderProtocol)> {
    if packet.len() < 40 {
        return None;
    } // invalid IPv6
    let ipv6_packet = Ipv6Packet::new(packet)?;
    let mut next_header = ipv6_packet.get_next_header();
    let mut offset = 40;

    loop {
        if offset + 2 > packet.len() {
            break;
        }
        let ext_header = &packet[offset..];
        next_header = IpNextHeaderProtocol(ext_header[0]);
        let ext_len = ext_header[1] as usize;
        offset += (ext_len + 1) * 8;
        if offset > packet.len() {
            return None;
        }
        if next_header == IpNextHeaderProtocols::Tcp || next_header == IpNextHeaderProtocols::Udp {
            break;
        };
    }

    Some((&packet[offset..], next_header))
}
