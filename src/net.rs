
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use option::TelnetOption;
use event::TelnetEvent;

pub struct TelnetStream<H> where H: Fn(TelnetEvent) {
    stream: TcpStream,
    options: Vec<TelnetOption>,
    event_handler: H
}

impl<H> TelnetStream<H> where H: Fn(TelnetEvent) {
    pub fn new(stream: TcpStream, options: Vec<TelnetOption>,
            event_handler: H) -> TelnetStream<H> {
        TelnetStream {
            stream: stream,
            options: options,
            event_handler: event_handler
        }
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, options: Vec<TelnetOption>,
            event_handler: H) -> io::Result<TelnetStream<H>> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(TelnetStream::new(stream, options, event_handler)),
            Err(e) => Err(e)
        }
    }
}

impl<H> Read for TelnetStream<H> where H: Fn(TelnetEvent) {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<H> Write for TelnetStream<H> where H: Fn(TelnetEvent) {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl<'a, H> Read for &'a TelnetStream<H> where H: Fn(TelnetEvent) {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut stream_ref: &TcpStream = &self.stream;
        stream_ref.read(buf)
    }
}

impl<'a, H> Write for &'a TelnetStream<H> where H: Fn(TelnetEvent) {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut stream_ref: &TcpStream = &self.stream;
        stream_ref.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut stream_ref: &TcpStream = &self.stream;
        stream_ref.flush()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
