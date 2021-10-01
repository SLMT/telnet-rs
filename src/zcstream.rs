use crate::stream::Stream;

/// Stream with ability to be upgraded to zlib stream.
pub trait ZCStream: Stream {
    /// Begin zlib decompression on downstream. Ignored if already enabled.
    fn begin_zlib(&mut self);
    /// Stop zlib decompression on downstream. Ignored if already disabled.
    fn end_zlib(&mut self);
}
