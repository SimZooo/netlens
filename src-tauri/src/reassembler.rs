use std::{collections::BTreeMap, net::IpAddr};

use pnet::packet::{ethernet::EtherType, ip::IpNextHeaderProtocol};

use crate::{
    parsers::parser::{ParseResult, ParseResultType},
    protocols::ProtocolId,
    Field, Layer, OsiLayer,
};

pub type FragmentedPackets = BTreeMap<FragmentedKey, IpFragmentedPacket>;

#[derive(Default, Debug)]
pub struct IpFragmentedPacket {
    pub done: bool,
    pub ethertype: Option<EtherType>,
    pub next_protocol: Option<IpNextHeaderProtocol>,
    pub fragments: BTreeMap<usize, Vec<u8>>,
    pub fields: Vec<Field>,
    pub ids: Vec<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FragmentedKey {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub protocol: IpNextHeaderProtocol,
    pub identification: u16,
}

impl IpFragmentedPacket {
    /// Create new fragmented packet with the first fragment
    pub fn new_first(
        done: bool,
        next_protocol: IpNextHeaderProtocol,
        ethertype: EtherType,
        fragment: Vec<u8>,
        byte_offset: usize,
        fields: Vec<Field>,
        id: String,
    ) -> Self {
        let mut fragments = BTreeMap::new();
        fragments.insert(byte_offset, fragment);

        Self {
            done,
            ethertype: Some(ethertype),
            next_protocol: Some(next_protocol),
            fragments,
            fields,
            ids: vec![id],
        }
    }
}

pub struct ReassembleResult {
    pub parse_result: ParseResult,
    pub fragment_ids: Vec<String>,
    pub defragmented_id: String,
}

pub struct Reassembler {}

impl Reassembler {
    pub fn update(fragmented_packets: &mut FragmentedPackets) -> Vec<ReassembleResult> {
        let mut results = vec![];
        let done_ids: Vec<FragmentedKey> = fragmented_packets
            .iter()
            .filter_map(|(id, packet)| packet.done.then_some(*id))
            .collect();

        for id in done_ids {
            let Some(frag) = fragmented_packets.remove(&id) else {
                println!("Failed to remove fragmented packet");
                continue;
            };

            let mut fields = frag.fields;

            let (last_offset, last_frag) = frag.fragments.iter().last().unwrap();
            let buffer_len = last_offset * 8 + last_frag.len();
            let mut buffer = vec![0u8; buffer_len];
            let n = frag.fragments.len();
            for (offset, data) in frag.fragments {
                let start = offset;
                let end = start + data.len();
                buffer[start..end].copy_from_slice(&data);
            }

            // Todo: keep reference to fragments and link IDs between defragmented and fragments
            let next = frag
                .next_protocol
                .and_then(|p| Some(ProtocolId::from_ip(p)))
                .unwrap_or(ProtocolId::None);
            fields.push(Field::new(
                "Defragmented".to_string(),
                format!("{} Fragments", n),
            ));
            results.push(ReassembleResult {
                parse_result: ParseResult {
                    layer: Layer {
                        name: "Internet Protocol Version 4".to_string(),
                        protocol: next.to_string(),
                        osi_layer: OsiLayer::Network,
                        fields,
                    },
                    next,
                    remaining: buffer,
                    result_type: ParseResultType::Normal,
                },
                fragment_ids: frag.ids,
                defragmented_id: uuid::Uuid::new_v4().to_string(),
            });
        }

        results
    }
}
