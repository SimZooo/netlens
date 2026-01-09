use crate::{protocols::ProtocolId, reassembler::FragmentedPackets, Layer};

pub struct ParseInput<'a> {
    pub bytes: &'a [u8],
}

pub struct ParseResult {
    pub layer: Layer,
    pub next: ProtocolId,
    pub remaining: Vec<u8>,
}

pub enum ParseOutcome {
    Continue(ProtocolId),
    Stop,
    Fragmented,
}

pub trait LayerParser {
    fn parse(
        input: &[u8],
        fragmented_packets: Option<&mut FragmentedPackets>,
    ) -> Option<ParseResult>;
}
