use std::io;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};

use net::TelnetStream;
use event::{TelnetEvent, TelnetEventQueue};
use negotiation::{NegotiationAction, NegotiationSM};
use option::{TelnetOption, TelnetOptionConfigs};
use byte::*;

#[derive(Debug)]
enum ProcessState {
    NormalData,
    IAC,
    SB,
    SBData(TelnetOption, usize),    // (option, start location of option data)
    SBDataIAC(TelnetOption, usize), // (option, start location of option data)
    Will, Wont,
    Do, Dont
}

pub struct TelnetConnection {
    stream: TelnetStream,
    option_configs: TelnetOptionConfigs,
    event_queue: TelnetEventQueue,

    // Buffer
    buffer: Box<[u8]>,
    buffered_size: usize,
    process_buffer: Box<[u8]>,
    process_buffered_size: usize,

    // Negotiation
    negotiation_sm: NegotiationSM
}

impl TelnetConnection {
    pub fn new(tcp_stream: TcpStream, option_configs: TelnetOptionConfigs,
            buf_size: usize) -> TelnetConnection {
        // Make sure the buffer size always >= 1
        let actual_size = if buf_size == 0 { 1 } else { buf_size };

        TelnetConnection {
            stream: TelnetStream::new(tcp_stream),
            option_configs: option_configs,
            event_queue: TelnetEventQueue::new(),
            buffer: vec![0; actual_size].into_boxed_slice(),
            buffered_size: 0,
            process_buffer: vec![0; actual_size].into_boxed_slice(),
            process_buffered_size: 0,
            negotiation_sm: NegotiationSM::new()
        }
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, option_configs: TelnetOptionConfigs,
            buf_size: usize) -> io::Result<TelnetConnection> {
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
                            let data_end = current;
                            let data = self.copy_buffered_data(data_start, data_end);
                            self.event_queue.push_event(TelnetEvent::DataReceived(data));
                        }
                    } else if current == self.buffered_size - 1 {
                        // It reaches the end of the buffer
                        let data_end = self.buffered_size;
                        let data = self.copy_buffered_data(data_start, data_end);
                        self.event_queue.push_event(TelnetEvent::DataReceived(data));
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
                        // Escaping
                        // TODO: Write a test case for this
                        BYTE_IAC => {
                            // Copy the data to the process buffer
                            self.append_data_to_proc_buffer(data_start, current - 1);

                            // Add escaped IAC
                            self.process_buffer[self.process_buffered_size] = BYTE_IAC;
                            self.process_buffered_size += 1;

                            // Update the state
                            state = ProcessState::NormalData;
                            data_start = current + 1;
                        },
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
                    let is_him_allowed = self.option_configs.is_him_allowed(&opt);

                    match state {
                        ProcessState::Will => {
                            self.event_queue.push_event(
                                TelnetEvent::NegotiationReceived(
                                    NegotiationAction::Will, opt));
                            self.negotiation_sm.receive_will(&mut self.event_queue,
                                &self.stream, opt, is_him_allowed);
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
                            self.negotiation_sm.receive_do(&mut self.event_queue,
                                &self.stream, opt, is_him_allowed);
                        },
                        ProcessState::Dont => {
                            self.event_queue.push_event(
                                TelnetEvent::NegotiationReceived(
                                    NegotiationAction::Dont, opt));
                            self.negotiation_sm.receive_dont(&mut self.event_queue,
                                &self.stream, opt);
                        },
                        _ => {} // Do nothing
                    }

                    state = ProcessState::NormalData;
                    data_start = current + 1;
                },

                // Start subnegotiation
                ProcessState::SB => {
                    let opt = TelnetOption::parse(byte);
                    state = ProcessState::SBData(opt, current + 1);
                },

                // Subnegotiation's data
                ProcessState::SBData(opt, data_start) => {
                    if byte == BYTE_IAC {
                        state = ProcessState::SBDataIAC(opt, data_start);
                    }

                    // XXX: We may need to consider the case that a SB Data
                    // sequence may exceed this buffer
                },

                // IAC inside Subnegotiation's data
                ProcessState::SBDataIAC(opt, sb_data_start) => {
                    match byte {
                        // The end of subnegotiation
                        BYTE_SE => {
                            // Update state
                            state = ProcessState::NormalData;
                            data_start = current + 1;

                            // Return the option
                            let sb_data_end = current - 1;
                            let data = self.copy_buffered_data(sb_data_start, sb_data_end);
                            self.event_queue.push_event(TelnetEvent::SBReceived(opt, data));
                        },
                        // Escaping
                        // TODO: Write a test case for this
                        BYTE_IAC => {
                            // Copy the data to the process buffer
                            self.append_data_to_proc_buffer(sb_data_start, current - 1);

                            // Add escaped IAC
                            self.process_buffer[self.process_buffered_size] = BYTE_IAC;
                            self.process_buffered_size += 1;

                            // Update the state
                            state = ProcessState::SBData(opt, current + 1);
                        },
                        // TODO: Write a test case for this
                        b => {
                            self.event_queue.push_event(TelnetEvent::Error(
                                format!("Unexpected byte after IAC inside SB: {}", b)));

                            // Copy the data to the process buffer
                            self.append_data_to_proc_buffer(sb_data_start, current - 1);
                            // Update the state
                            state = ProcessState::SBData(opt, current + 1);
                        }
                    }
                }
            }

            // Move to the next byte
            current += 1;
        }
    }

    // Copy the data to the process buffer
    fn append_data_to_proc_buffer(&mut self, data_start: usize, data_end: usize) {
        let data_length = data_end - data_start;
        let dst_start = self.process_buffered_size;
        let dst_end = self.process_buffered_size + data_length;
        let dst = &mut self.process_buffer[dst_start .. dst_end];
        dst.copy_from_slice(&self.buffer[data_start .. data_end]);
        self.process_buffered_size += data_length;
    }

    fn copy_buffered_data(&mut self, data_start: usize, data_end: usize) -> Box<[u8]> {
        let data = if self.process_buffered_size > 0 {
            // Copy the data to the process buffer
            self.append_data_to_proc_buffer(data_start, data_end);

            let pbe = self.process_buffered_size;
            self.process_buffered_size = 0;

            &self.process_buffer[0 .. pbe]
        } else {
            &self.buffer[data_start .. data_end]
        };

        Box::from(data)
    }
}
