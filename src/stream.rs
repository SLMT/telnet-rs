use std::{
    io::{Read, Result, Write},
    net::TcpStream,
    time::Duration,
};

#[allow(clippy::missing_errors_doc)]
pub trait Stream: Read + Write {
    fn set_nonblocking(&self, nonblocking: bool) -> Result<()>;
    fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()>;
}

impl Stream for TcpStream {
    fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        self.set_nonblocking(nonblocking)
    }

    fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()> {
        self.set_read_timeout(dur)
    }
}
