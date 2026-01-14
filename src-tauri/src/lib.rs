use std::sync::{Arc, Mutex};

use std::thread;
use std::time::Duration;

use pnet::datalink::{self, Channel::Ethernet};

use serde::Serialize;
use tauri::AppHandle;
use tauri::{Manager, State};

use crate::packet_handler::PacketHandler;

pub struct AppState {
    interface: Option<String>,
    listen: bool,
}

mod packet_handler;
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
        let mut packet_handler = PacketHandler::new();

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
                Ok(packet) => packet_handler.handle_packet(packet, app.clone()),
                Err(e) => eprintln!("{e}"),
            }
        }
    });
}
