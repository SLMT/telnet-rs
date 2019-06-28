use stream::Stream;

pub trait ZCStream: Stream {
    fn begin_zlib(&mut self);
    fn end_zlib(&mut self);
}
