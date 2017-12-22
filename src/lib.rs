
mod negotiation;
mod option;
mod event;
mod net;
mod byte;
mod connection;

pub use option::{TelnetOption, TelnetOptionConfigs};
pub use event::TelnetEvent;
pub use negotiation::NegotiationAction;
pub use connection::TelnetConnection;

pub struct Telnet {

}

impl Telnet {
    pub fn connect<A: ToSocketAddrs>(addr: A, buf_size: usize) -> io::Result<Telnet> {

    }

    pub fn read(&mut self) -> TelnetEvent {

    }

    pub fn read_timeout(&mut self, timeout: Duration) -> TelnetEvent {

    }

    pub fn read_nonblocking(&mut self) -> TelnetEvent {

    }

    pub fn write(&mut self, data: &[u8]) {
        
    }

    pub fn negotiate(&self, action: NegotiationAction, opt: TelnetOption) {

    }

    pub fn subnegotiate(&self, action: NegotiationAction, opt: TelnetOption, data: &[u8]) {

    }

    pub fn set_debug_logging(&mut self, enabled: bool) {

    }
}
