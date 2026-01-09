use std::collections::BTreeMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::EtherType;

use pnet::packet::ip::IpNextHeaderProtocol;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri::{Manager, State};

use crate::protocols::{ProtocolId, PROTOCOLS};
use crate::reassembler::{FragmentedKey, FragmentedPackets, Reassembler};

pub struct AppState {
    interface: Option<String>,
    listen: bool,
}

mod parsers;
mod protocols;
mod reassembler;

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

#[derive(Debug, Clone, Serialize, PartialEq)]
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
                thread::sleep(Duration::from_millis(50));
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
            for packet in reassembled_packets {
                let p = handle_packet(&packet.remaining, &mut fragmented_packets);
                if let Some(p) = p {
                    let _ = app.emit("packet_received", p);
                }
            }
        }
    });
}

pub fn handle_packet(
    packet: &[u8],
    fragmented_packets: &mut FragmentedPackets,
) -> Option<NetworkPacket> {
    let mut layers = vec![];
    let mut next = ProtocolId::Ethernet;
    let mut bytes = packet.to_vec();

    while next != ProtocolId::None {
        for protocol in PROTOCOLS {
            if protocol.name != next {
                continue;
            }

            let parser = protocol.parser?;

            let prot_res = (parser)(&bytes, Some(fragmented_packets));
            if let Some(parse_res) = prot_res {
                layers.push(parse_res.layer);
                bytes = parse_res.remaining.to_vec();
                next = parse_res.next;
            } else {
                next = ProtocolId::None;
            }
        }
    }

    // Prefer Network layer, fallback to DataLink
    let mut src = "".to_string();
    let mut dst = "".to_string();
    for layer in layers.iter().filter(|l| l.osi_layer == OsiLayer::Network) {
        for field in &layer.fields {
            match field.name.as_str() {
                "Source" | "src" => src = field.value.clone(),
                "Destination" | "dst" => dst = field.value.clone(),
                _ => {}
            }
        }
    }

    if src.is_empty() || dst.is_empty() {
        for layer in layers.iter().filter(|l| l.osi_layer == OsiLayer::DataLink) {
            for field in &layer.fields {
                match field.name.as_str() {
                    "Source" | "src" => src = field.value.clone(),
                    "Destination" | "dst" => dst = field.value.clone(),
                    _ => {}
                }
            }
        }
    }

    Some(NetworkPacket {
        src,
        dst,
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
