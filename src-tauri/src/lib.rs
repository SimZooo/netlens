use std::collections::BTreeMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EtherType, EtherTypes};

use pnet::packet::ip::IpNextHeaderProtocol;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri::{Manager, State};

use parsers::datalink_parser::EthernetParser;
use parsers::parser::{LayerParser, PacketContext};

use crate::parsers::network_parser::{Ipv4Parser, Ipv6Parser};
use crate::parsers::transport_parser::TcpParser;
use crate::protocols::{ProtocolNames, PROTOCOLS};
use crate::reassembler::{FragmentedKey, Reassembler};

pub struct AppState {
    interface: Option<String>,
    listen: bool,
}

mod parsers;
mod protocols;
mod reassembler;

pub type FragmentedPackets = BTreeMap<FragmentedKey, IpFragmentedPacket>;

#[derive(Default, Debug)]
pub struct IpFragmentedPacket {
    done: bool,
    src_ip: Option<IpAddr>,
    dst_ip: Option<IpAddr>,
    ethertype: Option<EtherType>,
    next_protocol: Option<IpNextHeaderProtocol>,
    fragments: BTreeMap<usize, Vec<u8>>,
}

impl IpFragmentedPacket {
    /// Create new fragmented packet with the first fragment
    fn new_first(
        src_ip: IpAddr,
        dst_ip: IpAddr,
        done: bool,
        next_protocol: IpNextHeaderProtocol,
        ethertype: EtherType,
        fragment: Vec<u8>,
        byte_offset: usize,
    ) -> Self {
        let mut fragments = BTreeMap::new();
        fragments.insert(byte_offset, fragment);

        Self {
            src_ip: Some(src_ip),
            dst_ip: Some(dst_ip),
            done,
            ethertype: Some(ethertype),
            next_protocol: Some(next_protocol),
            fragments,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Default)]
pub enum OsiLayer {
    #[default]
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
    pub protocol: String,
    pub name: String,
    pub osi_layer: OsiLayer,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkPacket {
    pub src: String,
    pub dst: String,
    pub id: String,
    pub timestamp: f64,
    pub raw: Vec<u8>,
    pub layers: Vec<Layer>,
    pub length: usize,
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
        let mut fragmented_packets: FragmentedPackets = BTreeMap::new();

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

            let reassembled_packets = Reassembler::update(&mut fragmented_packets);
            for mut packet in reassembled_packets {
                let Some(transport_layer) = TcpParser::parse(&vec![], &mut packet, None) else {
                    println!("Invalid transport layer in fragmented packet");
                    continue;
                };

                let _ = app.emit(
                    "packet_received",
                    NetworkPacket {
                        src: packet.src,
                        dst: packet.dst,
                        id: uuid::Uuid::new_v4().to_string(),
                        layers: transport_layer,
                        raw: packet.network_payload,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .and_then(|t| Ok(t.as_secs_f64()))
                            .unwrap_or(0.),
                        length: 0,
                    },
                );
            }
        }
    });
}

pub fn handle_packet(
    packet: &[u8],
    fragmented_packets: &mut FragmentedPackets,
) -> Option<NetworkPacket> {
    let mut layers = vec![];
    let mut packet_context = PacketContext::default();
    let datalink_layers =
        EthernetParser::parse(&packet.to_vec(), &mut packet_context, None).unwrap();

    for layer in datalink_layers {
        layers.push(layer);
    }

    let mut last_protocol = "".to_string();
    loop {
        let mut matched = false;
        for protocol in PROTOCOLS {
            if protocol.name.to_string() == packet_context.next_protocol {
                matched = true;
                if let Some(protocol_parser) = protocol.parser {
                    let prot_layers =
                        (protocol_parser)(&vec![], &mut packet_context, Some(fragmented_packets));
                    for layer in prot_layers.unwrap_or(vec![]) {
                        if protocol.name == ProtocolNames::Arp {
                            println!("yessir");
                        }
                        layers.push(layer);
                    }
                }
                break;
            }
        }
        if !matched || last_protocol == packet_context.next_protocol {
            break;
        }

        last_protocol = packet_context.next_protocol.clone();
    }

    Some(NetworkPacket {
        src: packet_context.src,
        dst: packet_context.dst,
        id: uuid::Uuid::new_v4().to_string(),
        layers,
        raw: packet.to_vec(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .and_then(|t| Ok(t.as_secs_f64()))
            .unwrap_or(0.),
        length: packet.len(),
    })
}
