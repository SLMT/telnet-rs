
use std::io;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};

use option::TelnetOption;
use event::TelnetEvent;

const BYTE_IAC: u8 = 255;   // interpret as command:
const BYTE_DONT: u8 = 254;  // you are not to use option
const BYTE_DO: u8 = 253;    // please, you use option
const BYTE_WONT: u8 = 252;  // I won't use option
const BYTE_WILL: u8 = 251;  // I will use option
const BYTE_SB: u8 = 250;    // interpret as subnegotiation
const BYTE_SE: u8 = 240;    // end sub negotiation

enum ProcessState {
    NormalData,
    IAC,
    SB,
    SBData(u8, usize),    // (option byte, start location of option data)
    SBDataIAC(u8, usize), // (option byte, start location of option data)
    WILL, WONT,
    DO, DONT
}

pub struct TelnetStream {
    stream: TcpStream,
    options: Vec<TelnetOption>,
    state: ProcessState,

    // Buffer
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
            state: ProcessState::NormalData,
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

        // Process the buffered bytes
        return Ok(self.process());
    }

    // TODO: Unhandled cases:
    // IAC IAC
    // IAC SB IAC
    fn process<'a>(&'a mut self) -> TelnetEvent<'a> {
        let mut current = self.next_start;

        while current < self.buffered_size {
            // Gather a byte
            let byte = self.buffer[current];

            // Process the byte
            match self.state {
                // Normal Data
                ProcessState::NormalData => {
                    if byte == BYTE_IAC {
                        // The following bytes will be commands

                        // Update the states
                        self.state = ProcessState::IAC;
                        let start = self.next_start;
                        self.next_start = current + 1;

                        // Send the data before this byte
                        if current > start {
                            return TelnetEvent::DataReceived(
                                &self.buffer[start .. current]);
                        }
                    } else if current == self.buffered_size - 1 {
                        // It reaches the end of the buffer
                        let event = TelnetEvent::DataReceived(
                            &self.buffer[self.next_start .. self.buffered_size]);
                        self.next_start = current + 1;

                        return event;
                    }
                },

                // Telnet Commands
                ProcessState::IAC => {
                    match byte {
                        // Negotiation Commands
                        BYTE_WILL => self.state = ProcessState::WILL,
                        BYTE_WONT => self.state = ProcessState::WONT,
                        BYTE_DO => self.state = ProcessState::DO,
                        BYTE_DONT => self.state = ProcessState::DONT,
                        // Subnegotiation
                        BYTE_SB => self.state = ProcessState::SB,
                        // Unknown IAC commands
                        _ => {
                            self.state = ProcessState::NormalData;
                            self.next_start = current + 1;
                            return TelnetEvent::UnknownIAC(byte);
                        }
                    }
                },

                // Negotiation
                ProcessState::WILL | ProcessState::WONT |
                        ProcessState::DO | ProcessState::DONT => {
                     self.state = ProcessState::NormalData;
                     self.next_start = current + 1;
                     // TODO: We should handle each case separately
                     return TelnetEvent::Negotiation(byte);
                 },

                // Start subnegotiation
                ProcessState::SB => {
                    self.state = ProcessState::SBData(byte, current + 1);
                },

                // Subnegotiation's data
                ProcessState::SBData(opt, data_start) => {
                    if byte == BYTE_IAC {
                        self.state = ProcessState::SBDataIAC(opt, data_start);
                    }
                },

                // IAC inside Subnegotiation's data
                ProcessState::SBDataIAC(opt, data_start) => {
                    if byte == BYTE_SE {
                        // Update state
                        self.state = ProcessState::NormalData;
                        self.next_start = current + 1;

                        // Return the option
                        let data_end = current - 1;
                        // TODO: We should handle each case separately
                        return TelnetEvent::Subnegotiation(opt,
                                &self.buffer[data_start .. data_end]);
                    }
                },

                // TODO: others
                _ => {

                }
            }

            // Move to the next byte
            current += 1;
        }

        return TelnetEvent::Nothing;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
