use std::net::IpAddr;

use pnet::packet::ip::IpNextHeaderProtocol;

use crate::{parsers::parser::PacketContext, FragmentedPackets, IpFragmentedPacket};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FragmentedKey {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub protocol: IpNextHeaderProtocol,
    pub identification: u16,
}

pub struct Reassembler {}

impl Reassembler {
    pub fn update(fragmented_packets: &mut FragmentedPackets) -> Vec<PacketContext> {
        let mut ctxs = vec![];
        let done_ids: Vec<FragmentedKey> = fragmented_packets
            .iter()
            .filter_map(|(id, packet)| packet.done.then_some(*id))
            .collect();

        for id in done_ids {
            let Some(frag) = fragmented_packets.remove(&id) else {
                println!("Failed to remove fragmented packet");
                continue;
            };

            let (last_offset, last_frag) = frag.fragments.iter().last().unwrap();
            let buffer_len = last_offset * 8 + last_frag.len();
            let mut buffer = vec![0u8; buffer_len];
            for (offset, data) in frag.fragments {
                let start = offset;
                let end = start + data.len();
                buffer[start..end].copy_from_slice(&data);
            }

            let mut packet_context = PacketContext::default();
            packet_context.network_payload = buffer;
            packet_context.ethertype = frag.ethertype;
            packet_context.next_protocol = frag.next_protocol.unwrap().to_string();

            // Todo: keep reference to fragments and link IDs between defragmented and fragments

            ctxs.push(packet_context);
        }

        ctxs
    }
}
