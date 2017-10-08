
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use option::TelnetOption;

pub struct TelnetStream {
    stream: TcpStream,
    options: Vec<TelnetOption>
}

impl TelnetStream {
    pub fn new(stream: TcpStream, options: Vec<TelnetOption>) -> TelnetStream {
        TelnetStream {
            stream: stream,
            options: options
        }
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, options: Vec<TelnetOption>) -> io::Result<TelnetStream> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(TelnetStream::new(stream, options)),
            Err(e) => Err(e)
        }
    }
}

impl Read for TelnetStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for TelnetStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl<'a> Read for &'a TelnetStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut stream_ref: &TcpStream = &self.stream;
        stream_ref.read(buf)
    }
}

impl<'a> Write for &'a TelnetStream {
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
