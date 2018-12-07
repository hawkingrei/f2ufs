use std::fs::File;

#[cfg(feature = "zstd")]
use zstd::block::decompress;

use super::Pio;

use super::*;

pub(crate) trait LogReader {
    fn read_segment_header(
        &self,
        id: LogId,
    ) -> Result<SegmentHeader, ()>;

    fn read_segment_trailer(
        &self,
        id: LogId,
    ) -> Result<SegmentTrailer, ()>;

    fn read_message_header(
        &self,
        id: LogId,
    ) -> Result<MessageHeader, ()>;

    fn read_message(
        &self,
        lid: LogId,
        config: &Config,
    ) -> Result<LogRead, ()>;
}