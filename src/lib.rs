
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct TelnetStream {
    stream: TcpStream
}

impl TelnetStream {
    pub fn new(stream: TcpStream) -> TelnetStream {
        TelnetStream {
            stream: stream
        }
    }

    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TelnetStream> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(TelnetStream::new(stream)),
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
