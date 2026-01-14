use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Emitter};

use crate::{
    protocols::{ProtocolId, PROTOCOLS},
    reassembler::{FragmentedPackets, Reassembler},
    NetworkPacket, OsiLayer,
};

pub const PACKET_RECEIVED: &'static str = "packet_received";
pub const FRAGMENTS_UPDATE: &'static str = "fragments_update";

pub struct PacketHandler {
    fragmented_packets: FragmentedPackets,
}

impl PacketHandler {
    pub fn new() -> Self {
        Self {
            fragmented_packets: BTreeMap::new(),
        }
    }

    pub fn handle_packet(&mut self, packet: &[u8], app: AppHandle) {
        let network_packet = Self::handle(packet, &mut self.fragmented_packets);
        if let Some(p) = network_packet {
            let _ = app.emit(PACKET_RECEIVED, p);
        }

        // TODO: Somehow make part of main loop
        let reassembled_packets = Reassembler::update(&mut self.fragmented_packets);
        for packet in reassembled_packets {
            let p =
                PacketHandler::handle(&packet.parse_result.remaining, &mut self.fragmented_packets);
            if let Some(p) = p {
                let _ = app.emit(PACKET_RECEIVED, p);
                let _ = app.emit(
                    FRAGMENTS_UPDATE,
                    (packet.fragment_ids, packet.defragmented_id),
                );
            } else {
                println!("Failed fragment");
            }
        }
    }

    fn handle(packet: &[u8], fragmented_packets: &mut FragmentedPackets) -> Option<NetworkPacket> {
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
}
