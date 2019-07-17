
//! #### MCCP2
//! A feature of some telnet servers is `MCCP2` which allows the downstream data to be compressed.
//! To use this, first enable the `zcstream` [rust feature](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section) for this crate.
//! Then in the code deal with the relevant events, and switch the zlib when appropriate.
//!
//! Basic usage example:
//! ```
//! match event {
//! 	TelnetEvent::Data(buffer) => {
//! 		println!("{}", &std::str::from_utf8(&(*buffer)).unwrap());
//! 	},
//! 	TelnetEvent::Negotiation(NegotiationAction::Will, TelnetOption::Compress2) => {
//! 		telnet.negotiate(NegotiationAction::Do, TelnetOption::Compress2);
//! 	},
//! 	TelnetEvent::Subnegotiation(TelnetOption::Compress2, _) => {
//! 		telnet.begin_zlib();
//! 	}
//! }
//! ```
mod negotiation;
mod option;
mod event;
mod byte;
mod stream;
#[cfg(feature = "zcstream")]
mod zcstream;
#[cfg(feature = "zcstream")]
mod zlibstream;

pub use stream::Stream;
#[cfg(feature = "zcstream")]
pub use zlibstream::ZlibStream;
#[cfg(feature = "zcstream")]
pub use zcstream::ZCStream;
pub use option::TelnetOption;
pub use event::TelnetEvent;
pub use negotiation::NegotiationAction;

use std::io;
use std::io::{Read, Write, ErrorKind};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use event::TelnetEventQueue;
use byte::*;

#[cfg(feature = "zcstream")]
type TStream = zcstream::ZCStream;
#[cfg(not(feature = "zcstream"))]
type TStream = stream::Stream;

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

///
/// A telnet connection to a remote host.
///
/// # Examples
/// ```rust,should_panic
/// use telnet::Telnet;
///
/// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
///         .expect("Couldn't connect to the server...");
/// loop {
///     let event = connection.read().expect("Read Error");
///     println!("{:?}", event);
/// }
/// ```
///
pub struct Telnet {
    stream: Box<TStream>,
    event_queue: TelnetEventQueue,

    // Buffer
    buffer: Box<[u8]>,
    buffered_size: usize,
    process_buffer: Box<[u8]>,
    process_buffered_size: usize
}

impl Telnet {

    ///
    /// Opens a telnet connection to a remote host using a `TcpStream`.
    ///
    /// `addr` is an address of the remote host. Note that a remote host usually opens port 23 for
    /// a Telnet connection. `buf_size` is a size of the underlying buffer for processing the data
    ///  read from the remote host.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::Telnet;
    ///
    /// let connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// ```
    ///
    pub fn connect<A: ToSocketAddrs>(addr: A, buf_size: usize) -> io::Result<Telnet> {
        let stream = TcpStream::connect(addr)?; // send the error out directly

        #[cfg(feature = "zcstream")]
        return Ok(Telnet::from_stream(Box::new(ZlibStream::from_stream(stream)), buf_size));
        #[cfg(not(feature = "zcstream"))]
        return Ok(Telnet::from_stream(Box::new(stream), buf_size));
    }

    #[cfg(feature = "zcstream")]
    pub fn begin_zlib(&mut self) {
        self.stream.begin_zlib()
    }

    #[cfg(feature = "zcstream")]
    pub fn end_zlib(&mut self) {
        self.stream.end_zlib()
    }
    /// Open a telnet connection to a remote host using a generic stream.
    /// 
    /// Communication will be made with the host using `stream`. `buf_size` is the size of the underlying
    /// buffer for processing data from the host.
    /// 
    /// Use this version of the constructor if you want to provide your own stream, for example if you want
    /// to mock out the remote host for testing purposes, or want to wrap the data the data with TLS encryption.
    pub fn from_stream(stream: Box<TStream>, buf_size: usize) -> Telnet {
        let actual_size = if buf_size == 0 { 1 } else { buf_size };

        Telnet {
            stream: stream,
            event_queue: TelnetEventQueue::new(),
            buffer: vec![0; actual_size].into_boxed_slice(),
            buffered_size: 0,
            process_buffer: vec![0; actual_size].into_boxed_slice(),
            process_buffered_size: 0
        }
    }

    ///
    /// Reads a `TelnetEvent`.
    ///
    /// If there was not any queued `TelnetEvent`, it would read a chunk of data into its buffer,
    /// extract any telnet command in the message, and queue all processed results. Otherwise, it
    /// would take a queued `TelnetEvent` without reading data from `TcpStream`.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::Telnet;
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// let event = connection.read().expect("Read Error");
    /// println!("{:?}", event);
    /// ```
    ///
    pub fn read(&mut self) -> io::Result<TelnetEvent> {
        while self.event_queue.is_empty() {
            // Set stream settings
            self.stream.set_nonblocking(false).expect("set_nonblocking call failed");
            self.stream.set_read_timeout(None).expect("set_read_timeout call failed");

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
        Ok(
            match self.event_queue.take_event() {
                Some(x) => x,
                None => TelnetEvent::Error("Internal Queue error".to_string())
            }
        )
    }

    ///
    /// Reads a `TelnetEvent`, but the waiting time cannot exceed a given `Duration`.
    ///
    /// This method is similar to `read()`, but with a time limitation. If the given time was
    /// reached, it would return `TelnetEvent::TimedOut`.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use std::time::Duration;
    /// use telnet::Telnet;
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// let event = connection.read_timeout(Duration::new(5, 0)).expect("Read Error");
    /// println!("{:?}", event);
    /// ```
    ///
    pub fn read_timeout(&mut self, timeout: Duration) -> io::Result<TelnetEvent> {
        if self.event_queue.is_empty() {
            // Set stream settings
            self.stream.set_nonblocking(false).expect("set_nonblocking call failed");
            self.stream.set_read_timeout(Some(timeout)).expect("set_read_timeout call failed");

            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer) {
                Ok(size) => {
                    self.buffered_size = size;
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                        return Ok(TelnetEvent::TimedOut);
                    } else {
                        return Err(e);
                    }
                }
            }

            self.process();
        }

        // Return an event
        Ok(
            match self.event_queue.take_event() {
                Some(x) => x,
                None => TelnetEvent::Error("Internal Queue error".to_string())
            }
        )
    }

    ///
    /// Reads a `TelnetEvent`. Return immediataly if there was no queued event and nothing to read.
    ///
    /// This method is a non-blocking version of `read()`. If there was no more data, it would
    /// return `TelnetEvent::NoData`.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::Telnet;
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// let event = connection.read_nonblocking().expect("Read Error");
    /// println!("{:?}", event);
    /// ```
    ///
    pub fn read_nonblocking(&mut self) -> io::Result<TelnetEvent> {
        if self.event_queue.is_empty() {
            // Set stream settings
            self.stream.set_nonblocking(true).expect("set_nonblocking call failed");
            self.stream.set_read_timeout(None).expect("set_read_timeout call failed");

            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer) {
                Ok(size) => {
                    self.buffered_size = size;
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        return Ok(TelnetEvent::NoData);
                    } else {
                        return Err(e);
                    }
                }
            }

            self.process();
        }

        // Return an event
        Ok(
            match self.event_queue.take_event() {
                Some(x) => x,
                None => TelnetEvent::Error("Internal Queue error".to_string())
            }
        )

    }

    ///
    /// Writes a given data block to the remote host. It will double any IAC byte.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::Telnet;
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// let buffer: [u8; 4] = [83, 76, 77, 84];
    /// connection.write(&buffer).expect("Write Error");
    /// ```
    ///
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut write_size = 0;

        let mut start = 0;
        for i in 0..data.len() {
            if data[i] == BYTE_IAC {
                self.stream.write(&data[start .. i + 1])?;
                self.stream.write(&[BYTE_IAC])?;
                write_size = write_size + (i + 1 - start);
                start = i + 1;
            }
        }

        if start < data.len() {
            self.stream.write(&data[start .. data.len()])?;
            write_size = write_size + (data.len() - start);
        }

        Ok(write_size)
    }

    ///
    /// Negotiates a telnet option with the remote host.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::{Telnet, NegotiationAction, TelnetOption};
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// connection.negotiate(NegotiationAction::Will, TelnetOption::Echo);
    /// ```
    ///
    pub fn negotiate(&mut self, action: NegotiationAction, opt: TelnetOption) {
        let buf: &[u8] = &[BYTE_IAC, action.to_byte(), opt.to_byte()];
        self.stream.write(buf).expect("Error sending negotiation");
    }

    ///
    /// Send data for sub-negotiation with the remote host.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::{Telnet, NegotiationAction, TelnetOption};
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// connection.negotiate(NegotiationAction::Do, TelnetOption::TTYPE);
    /// let data: [u8; 1] = [1];
    /// connection.subnegotiate(TelnetOption::TTYPE, &data);
    /// ```
    ///
    pub fn subnegotiate(&mut self, opt: TelnetOption, data: &[u8]) {
        let buf: &[u8] = &[BYTE_IAC, BYTE_SB, opt.to_byte()];
        self.stream.write(buf).expect("Error sending subnegotiation (START)");

        self.stream.write(data).expect("Error sending subnegotiation (DATA)");

        let buf: &[u8] = &[BYTE_IAC, BYTE_SE];
        self.stream.write(buf).expect("Error sending subnegotiation (END)");
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
                            self.event_queue.push_event(TelnetEvent::Data(data));

                            // Update the state
                            data_start = current;
                        }
                    } else if current == self.buffered_size - 1 {
                        // It reaches the end of the buffer
                        let data_end = self.buffered_size;
                        let data = self.copy_buffered_data(data_start, data_end);
                        self.event_queue.push_event(TelnetEvent::Data(data));
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

                    match state {
                        ProcessState::Will => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Will, opt));
                        },
                        ProcessState::Wont => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Wont, opt));
                        },
                        ProcessState::Do => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Do, opt));
                        },
                        ProcessState::Dont => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Dont, opt));
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
                            self.event_queue.push_event(TelnetEvent::Subnegotiation(opt, data));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Error;
    use std::ops::Deref;

    struct MockStream {
        test_data: Vec<u8>
    }

    impl MockStream {
        fn new(data: Vec<u8>) -> MockStream {
            MockStream {
                test_data: data,
            }
        }
    }

    impl stream::Stream for MockStream {
        fn set_nonblocking(&self, _nonblocking: bool) -> Result<(), Error> {
            return Ok(())
        }

        fn set_read_timeout(&self, _dur: Option<Duration>) -> Result<(), Error> {
            return Ok(())
        }
    }

    impl io::Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut offset = 0;
            while offset < buf.len() && offset < self.test_data.len() {
                buf[offset] = self.test_data[offset];
                offset += 1;
            }
            return Ok(offset);
        }
    }

    impl io::Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            return Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            return Ok(())
        }
    }

    #[test]
    fn escapes_double_iac_correctly() {
        let stream = Box::new(MockStream::new(vec!(0x40, 0x5a, 0xff, 0xff, 0x31, 0x34)));

        let mut telnet = Telnet::from_stream(stream, 6);

        let expected_bytes_1: [u8;2] = [0x40, 0x5a];
        let expected_bytes_2: [u8;3] = [0xff, 0x31, 0x34];

        let event_1 = telnet.read_nonblocking().unwrap();
        match event_1 {
            TelnetEvent::Data(buffer) => {
                assert_eq!(buffer.deref(), &expected_bytes_1);
            },
            _ => {
                assert!(false);
            }
        }

        let event_2 = telnet.read_nonblocking().unwrap();
        match event_2 {
            TelnetEvent::Data(buffer) => {
                assert_eq!(buffer.deref(), &expected_bytes_2);
            },
            _ => {
                assert!(false);
            }
        }
    }
}
