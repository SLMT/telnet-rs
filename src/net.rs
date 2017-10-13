
use std::io;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};

use option::TelnetOption;
use event::TelnetEvent;

pub struct TelnetStream {
    stream: TcpStream,
    options: Vec<TelnetOption>,
    buffer: Box<[u8]>,
    next_start: usize, // the position next start to read
    buffered_size: usize
}

impl TelnetStream {
    pub fn new(stream: TcpStream, options: Vec<TelnetOption>, buf_size: usize) -> TelnetStream {
        // Make sure the buffer size always >= 1
        let actual_size = if buf_size == 0 { 1 } else { buf_size };

        TelnetStream {
            stream: stream,
            options: options,
            buffer: vec![0; actual_size].into_boxed_slice(),
            next_start: 0,
            buffered_size: 0
        }
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, options: Vec<TelnetOption>, buf_size: usize) -> io::Result<TelnetStream> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(TelnetStream::new(stream, options, buf_size)),
            Err(e) => Err(e)
        }
    }

    pub fn read_event<'a>(&'a mut self) -> io::Result<TelnetEvent<'a>> {
        // No more data in the buffer to be processed
        if self.next_start >= self.buffered_size {
            match self.stream.read(&mut self.buffer) {
                Ok(size) => {
                    self.buffered_size = size;
                    self.next_start = 0;
                },
                Err(e) => return Err(e)
            }
        }

        // Return the data
        let start = self.next_start;
        let end = self.buffered_size;
        let event = TelnetEvent::DataReceived(&self.buffer[start .. end]);
        self.next_start = self.buffered_size;
        return Ok(event);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
