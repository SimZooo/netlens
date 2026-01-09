use tauri::App;

use crate::{parsers::parser::LayerParser, reassembler::FragmentedPackets};

pub struct ApplicationParser;

impl LayerParser for ApplicationParser {
    fn parse(
        input: &[u8],
        fragmented_packets: Option<&mut FragmentedPackets>,
    ) -> Option<super::parser::ParseResult> {
        None
    }
}
