use std::time::Duration;
use std::io::{Read, Write, Result};
use std::net::TcpStream;

pub trait Stream: Read + Write {
    fn set_nonblocking(&self, nonblocking: bool) -> Result<()>;
    fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()>;
}

impl Stream for TcpStream {
    fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        TcpStream::set_nonblocking(self, nonblocking)
    }

    fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()> {
        TcpStream::set_read_timeout(self, dur)
    }
}