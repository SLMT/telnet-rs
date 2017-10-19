
mod negotiation;
mod option;
mod event;
mod net;
mod byte;

pub use option::{TelnetOption, TelnetOptionConfig};
pub use event::TelnetEvent;
pub use negotiation::NegotiationAction;

use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};

use net::TelnetStream;
use event::TelnetEventQueue;
use negotiation::NegotiationSM;
use byte::*;

#[derive(Debug)]
enum ProcessState {
    NormalData,
    IAC,
    SB,
    SBData(u8, usize),    // (option byte, start location of option data)
    SBDataIAC(u8, usize), // (option byte, start location of option data)
    Will, Wont,
    Do, Dont
}

pub struct TelnetConnection {
    stream: TelnetStream,
    option_configs: HashMap<TelnetOption, TelnetOptionConfig>,
    event_queue: TelnetEventQueue,

    // Buffer
    buffer: Box<[u8]>,
    buffered_size: usize,

    // Negotiation
    negotiation_sm: NegotiationSM
}

impl TelnetConnection {
    pub fn new(tcp_stream: TcpStream, option_configs: HashMap<TelnetOption, TelnetOptionConfig>,
            buf_size: usize) -> TelnetConnection {
        // Make sure the buffer size always >= 1
        let actual_size = if buf_size == 0 { 1 } else { buf_size };

        TelnetConnection {
            stream: TelnetStream::new(tcp_stream),
            option_configs: option_configs,
            event_queue: TelnetEventQueue::new(),
            buffer: vec![0; actual_size].into_boxed_slice(),
            buffered_size: 0,
            negotiation_sm: NegotiationSM::new()
        }
    }

    pub fn connect<A: ToSocketAddrs>(addr: A,
            option_configs: HashMap<TelnetOption, TelnetOptionConfig>, buf_size: usize) ->
            io::Result<TelnetConnection> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(TelnetConnection::new(stream, option_configs, buf_size)),
            Err(e) => Err(e)
        }
    }

    pub fn read_event(&mut self) -> io::Result<TelnetEvent> {
        while self.event_queue.is_empty() {
            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer) {
                Ok(size) => {
                    self.buffered_size = size;
                },
                Err(e) => return Err(e)
            }

            self.process();
        }

        // Return an event
        Ok(self.event_queue.take_event().unwrap())
    }

    // TODO: Unhandled cases:
    // IAC IAC
    // IAC SB IAC
    fn process(&mut self) {
        let mut current = 0;
        let mut state = ProcessState::NormalData;
        let mut data_start = 0;

        while current < self.buffered_size {
            // Gather a byte
            let byte = self.buffer[current];

            // Process the byte
            match state {
                // Normal Data
                ProcessState::NormalData => {
                    if byte == BYTE_IAC {
                        // The following bytes will be commands

                        // Update the state
                        state = ProcessState::IAC;

                        // Send the data before this byte
                        if current > data_start {
                            self.event_queue.push_event(TelnetEvent::DataReceived(
                                Box::from(&self.buffer[data_start .. current])));
                        }
                    } else if current == self.buffered_size - 1 {
                        // It reaches the end of the buffer
                        self.event_queue.push_event(TelnetEvent::DataReceived(
                            Box::from(&self.buffer[data_start .. self.buffered_size])));
                    }
                },

                // Telnet Commands
                ProcessState::IAC => {
                    match byte {
                        // Negotiation Commands
                        BYTE_WILL => state = ProcessState::Will,
                        BYTE_WONT => state = ProcessState::Wont,
                        BYTE_DO => state = ProcessState::Do,
                        BYTE_DONT => state = ProcessState::Dont,
                        // Subnegotiation
                        BYTE_SB => state = ProcessState::SB,
                        // Unknown IAC commands
                        _ => {
                            state = ProcessState::NormalData;
                            data_start = current + 1;
                            self.event_queue.push_event(TelnetEvent::UnknownIAC(byte));
                        }
                    }
                },

                // Negotiation
                ProcessState::Will | ProcessState::Wont |
                        ProcessState::Do | ProcessState::Dont => {

                    let opt = TelnetOption::parse(byte);
                    let opt_config = match self.option_configs.get(&opt) {
                        Some(config) => config,
                        None => &TelnetOptionConfig { us: false, him: false }
                    };

                    match state {
                        ProcessState::Will => {
                            self.event_queue.push_event(
                                TelnetEvent::NegotiationReceived(
                                    NegotiationAction::Will, opt));
                            self.negotiation_sm.receive_will(&mut self.event_queue,
                                &self.stream, opt, opt_config);
                        },
                        ProcessState::Wont => {
                            self.event_queue.push_event(
                                TelnetEvent::NegotiationReceived(
                                    NegotiationAction::Wont, opt));
                            self.negotiation_sm.receive_wont(&mut self.event_queue,
                                &self.stream, opt);
                        },
                        ProcessState::Do => {
                            self.event_queue.push_event(
                                TelnetEvent::NegotiationReceived(
                                    NegotiationAction::Do, opt));
                            // TODO: Handle the negotiation
                        },
                        ProcessState::Dont => {
                            self.event_queue.push_event(
                                TelnetEvent::NegotiationReceived(
                                    NegotiationAction::Dont, opt));
                            // TODO: Handle the negotiation
                        },
                        _ => {} // Do nothing
                    }

                    state = ProcessState::NormalData;
                    data_start = current + 1;
                },

                // Start subnegotiation
                ProcessState::SB => {
                    state = ProcessState::SBData(byte, current + 1);
                },

                // Subnegotiation's data
                ProcessState::SBData(opt, data_start) => {
                    if byte == BYTE_IAC {
                        state = ProcessState::SBDataIAC(opt, data_start);
                    }
                },

                // IAC inside Subnegotiation's data
                ProcessState::SBDataIAC(opt, sb_data_start) => {
                    if byte == BYTE_SE {
                        // Update state
                        state = ProcessState::NormalData;
                        data_start = current + 1;

                        // Return the option
                        let sb_data_end = current - 1;
                        self.event_queue.push_event(TelnetEvent::SBReceived(
                            opt, Box::from(&self.buffer[sb_data_start .. sb_data_end])));
                    }
                },

                // TODO: others
                _ => {

                }
            }

            // Move to the next byte
            current += 1;
        }
    }
}
