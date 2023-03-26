//! #### MCCP2
//! A feature of some telnet servers is `MCCP2` which allows the downstream data to be compressed.
//! To use this, first enable the `zcstream` [rust feature](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section) for this crate.
//! Then in the code deal with the relevant events, and switch the zlib when appropriate.
//!
//! Basic usage example:
//! ```ignore
//! match event {
//!     Event::Data(buffer) => {
//!         println!("{}", &std::str::from_utf8(&(*buffer)).unwrap());
//!     },
//!     Event::Negotiation(Action::Will, TelnetOption::Compress2) => {
//!         telnet.negotiate(Action::Do, TelnetOption::Compress2);
//!     },
//!     Event::Subnegotiation(TelnetOption::Compress2, _) => {
//!         telnet.begin_zlib();
//!     }
//! }
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::upper_case_acronyms)]

mod byte;
mod error;
mod event;
mod negotiation;
mod option;
mod stream;
#[cfg(feature = "zcstream")]
mod zcstream;
#[cfg(feature = "zcstream")]
mod zlibstream;

// Re-exports
pub use error::{Error as TelnetError, SubnegotiationType};
pub use event::Event;
pub use negotiation::Action;
pub use option::TelnetOption;
pub use stream::Stream;
#[cfg(feature = "zcstream")]
pub use zcstream::ZCStream;
#[cfg(feature = "zcstream")]
pub use zlibstream::ZlibStream;

#[allow(clippy::wildcard_imports)]
use byte::*;
#[allow(clippy::enum_glob_use)]
use error::Error::*;
use event::TelnetEventQueue;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

#[cfg(feature = "zcstream")]
type TStream = dyn zcstream::ZCStream + Send + Sync;
#[cfg(not(feature = "zcstream"))]
type TStream = dyn stream::Stream + Send + Sync;

#[derive(Debug)]
enum ProcessState {
    NormalData,
    IAC,
    SB,
    SBData(TelnetOption, usize), // (option, start location of option data)
    SBDataIAC(TelnetOption, usize), // (option, start location of option data)
    Will,
    Wont,
    Do,
    Dont,
}

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
pub struct Telnet {
    stream: Box<TStream>,
    event_queue: TelnetEventQueue,

    // Buffer
    buffer: Box<[u8]>,
    buffered_size: usize,
    process_buffer: Box<[u8]>,
    process_buffered_size: usize,
}

#[allow(clippy::must_use_candidate)]
impl Telnet {
    /// Opens a telnet connection to a remote host using a [`TcpStream`].
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
    /// # Errors
    /// - Tcp connection failure
    pub fn connect<A: ToSocketAddrs>(addr: A, buf_size: usize) -> io::Result<Telnet> {
        let stream = TcpStream::connect(addr)?; // send the error out directly

        #[cfg(feature = "zcstream")]
        return Ok(Telnet::from_stream(
            Box::new(ZlibStream::from_stream(stream)),
            buf_size,
        ));
        #[cfg(not(feature = "zcstream"))]
        return Ok(Telnet::from_stream(Box::new(stream), buf_size));
    }
    /// Opens a telnet connection to a remote host using a TcpStream with a timeout [`Duration`]. Uses a [`TcpStream::connect_timeout`] under the hood
    /// and so can only be passed a single address of type [`SocketAddr`], and passing a zero [`Duration`] results in an error.
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::Telnet;
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// use std::str::FromStr;
    /// use std::time::Duration;
    /// let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str("127.0.0.1")
    ///                                 .expect("Invalid address")), 23);
    /// let telnet = Telnet::connect_timeout(&address, 256, Duration::from_secs(2))
    ///                                 .expect("Couldn't connect to the server...");
    /// ```
    ///
    /// # Errors
    /// - Tcp connection failure
    /// - I/O timeout error
    pub fn connect_timeout(
        addr: &SocketAddr,
        buf_size: usize,
        timeout: Duration,
    ) -> io::Result<Telnet> {
        let stream = TcpStream::connect_timeout(addr, timeout)?; // send the error out directly

        #[cfg(feature = "zcstream")]
        return Ok(Telnet::from_stream(
            Box::new(ZlibStream::from_stream(stream)),
            buf_size,
        ));
        #[cfg(not(feature = "zcstream"))]
        return Ok(Telnet::from_stream(Box::new(stream), buf_size));
    }

    #[cfg(feature = "zcstream")]
    pub fn begin_zlib(&mut self) {
        self.stream.begin_zlib();
    }

    #[cfg(feature = "zcstream")]
    pub fn end_zlib(&mut self) {
        self.stream.end_zlib();
    }

    /// Open a telnet connection to a remote host using a generic stream.
    ///
    /// Communication will be made with the host using `stream`. `buf_size` is the size of the underlying
    /// buffer for processing data from the host.
    ///
    /// Use this version of the constructor if you want to provide your own stream, for example if you want
    /// to mock out the remote host for testing purposes, or want to wrap the data with TLS encryption.
    pub fn from_stream(stream: Box<TStream>, buf_size: usize) -> Telnet {
        let actual_size = if buf_size == 0 { 1 } else { buf_size };

        Telnet {
            stream,
            event_queue: TelnetEventQueue::new(),
            buffer: vec![0; actual_size].into_boxed_slice(),
            buffered_size: 0,
            process_buffer: vec![0; actual_size].into_boxed_slice(),
            process_buffered_size: 0,
        }
    }

    /// Reads an [`Event`].
    ///
    /// If there was not any queued [`Event`], it would read a chunk of data into its buffer,
    /// extract any telnet command in the message, and queue all processed results. Otherwise, it
    /// would take a queued [`Event`] without reading data from [`TcpStream`].
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
    /// # Errors
    /// - Read stream fails
    /// - Set stream settings fails
    pub fn read(&mut self) -> io::Result<Event> {
        while self.event_queue.is_empty() {
            // Set stream settings
            self.stream.set_nonblocking(false)?;
            self.stream.set_read_timeout(None)?;

            // Read bytes to the buffer
            self.buffered_size = self.stream.read(&mut self.buffer)?;

            self.process();
        }

        // Return an event
        Ok(self
            .event_queue
            .take_event()
            .unwrap_or(Event::Error(InternalQueueErr)))
    }

    /// Reads an [`Event`], but the waiting time cannot exceed a given [`Duration`].
    ///
    /// This method is similar to [`Telnet::read`], but with a time limitation. If the given time was
    /// reached, it would return [`Event::TimedOut`].
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
    /// # Errors
    /// - Set stream settings fails
    /// - Read stream fails
    pub fn read_timeout(&mut self, timeout: Duration) -> io::Result<Event> {
        if self.event_queue.is_empty() {
            // Set stream settings
            self.stream.set_nonblocking(false)?;
            self.stream.set_read_timeout(Some(timeout))?;

            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer) {
                Ok(size) => self.buffered_size = size,
                Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                    return Ok(Event::TimedOut)
                }
                Err(e) => return Err(e),
            }

            self.process();
        }

        // Return an event
        Ok(self
            .event_queue
            .take_event()
            .unwrap_or(Event::Error(InternalQueueErr)))
    }

    /// Reads an [`Event`]. Returns immediately if there was no queued event and nothing to read.
    ///
    /// This method is a non-blocking version of [`Telnet::read`]. If there was no more data, it would
    /// return [`Event::NoData`].
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
    /// # Errors
    /// - Set stream settings fails
    /// - Read stream fails
    pub fn read_nonblocking(&mut self) -> io::Result<Event> {
        if self.event_queue.is_empty() {
            // Set stream settings
            self.stream.set_nonblocking(true)?;
            self.stream.set_read_timeout(None)?;

            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer) {
                Ok(size) => self.buffered_size = size,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(Event::NoData),
                Err(e) => return Err(e),
            }

            self.process();
        }

        // Return an event
        Ok(self
            .event_queue
            .take_event()
            .unwrap_or(Event::Error(InternalQueueErr)))
    }

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
    /// # Errors
    /// - Write to stream fails
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut write_size = 0;

        let mut start = 0;
        for i in 0..data.len() {
            if data[i] == BYTE_IAC {
                self.stream.write_all(&data[start..=i])?;
                self.stream.write_all(&[BYTE_IAC])?;
                write_size += i + 1 - start;
                start = i + 1;
            }
        }

        if start < data.len() {
            self.stream.write_all(&data[start..data.len()])?;
            write_size += data.len() - start;
        }

        Ok(write_size)
    }

    /// Negotiates a telnet option with the remote host.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::{Telnet, Action, TelnetOption};
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// connection.negotiate(&Action::Will, TelnetOption::Echo);
    /// ```
    ///
    /// # Errors
    /// - [`TelnetError::NegotiationErr`] if negotiation fails
    pub fn negotiate(&mut self, action: &Action, opt: TelnetOption) -> Result<(), TelnetError> {
        let buf = &[BYTE_IAC, action.as_byte(), opt.as_byte()];
        self.stream.write_all(buf).or(Err(NegotiationErr))?;
        Ok(())
    }

    /// Send data for sub-negotiation with the remote host.
    ///
    /// # Examples
    /// ```rust,should_panic
    /// use telnet::{Telnet, Action, TelnetOption};
    ///
    /// let mut connection = Telnet::connect(("127.0.0.1", 23), 256)
    ///         .expect("Couldn't connect to the server...");
    /// connection.negotiate(&Action::Do, TelnetOption::TTYPE);
    /// let data: [u8; 1] = [1];
    /// connection.subnegotiate(TelnetOption::TTYPE, &data);
    /// ```
    ///
    /// # Errors
    /// - [`TelnetError::SubnegotiationErr`] if subnegotiation fails
    #[allow(clippy::shadow_unrelated)]
    pub fn subnegotiate(&mut self, opt: TelnetOption, data: &[u8]) -> Result<(), TelnetError> {
        let buf = &[BYTE_IAC, BYTE_SB, opt.as_byte()];
        self.stream
            .write_all(buf)
            .or(Err(SubnegotiationErr(SubnegotiationType::Start)))?;

        self.stream
            .write_all(data)
            .or(Err(SubnegotiationErr(SubnegotiationType::Data)))?;

        let buf = &[BYTE_IAC, BYTE_SE];

        self.stream
            .write_all(buf)
            .or(Err(SubnegotiationErr(SubnegotiationType::End)))?;

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn process(&mut self) {
        let mut current = 0;
        let mut state = ProcessState::NormalData;
        let mut data_start = 0;

        while current < self.buffered_size {
            // Gather a byte
            let byte = self.buffer[current];

            match state {
                ProcessState::NormalData => {
                    if byte == BYTE_IAC {
                        // The following bytes will be commands
                        // Update the state
                        state = ProcessState::IAC;

                        // Send the data before this byte
                        if current > data_start {
                            let data_end = current;
                            let data = self.copy_buffered_data(data_start, data_end);
                            self.event_queue.push_event(Event::Data(data));

                            // Update the state
                            data_start = current;
                        }
                    } else if current == self.buffered_size - 1 {
                        // If it reaches the end of the buffer
                        let data_end = self.buffered_size;
                        let data = self.copy_buffered_data(data_start, data_end);
                        self.event_queue.push_event(Event::Data(data));
                    }
                }

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
                        }
                        // Unknown IAC commands
                        _ => {
                            state = ProcessState::NormalData;
                            data_start = current + 1;
                            self.event_queue.push_event(Event::UnknownIAC(byte));
                        }
                    }
                }

                // Negotiation
                ProcessState::Will | ProcessState::Wont | ProcessState::Do | ProcessState::Dont => {
                    let opt = TelnetOption::parse(byte);

                    match state {
                        ProcessState::Will => {
                            self.event_queue
                                .push_event(Event::Negotiation(Action::Will, opt));
                        }
                        ProcessState::Wont => {
                            self.event_queue
                                .push_event(Event::Negotiation(Action::Wont, opt));
                        }
                        ProcessState::Do => {
                            self.event_queue
                                .push_event(Event::Negotiation(Action::Do, opt));
                        }
                        ProcessState::Dont => {
                            self.event_queue
                                .push_event(Event::Negotiation(Action::Dont, opt));
                        }
                        _ => {} // Do nothing
                    }

                    state = ProcessState::NormalData;
                    data_start = current + 1;
                }

                // Start subnegotiation
                ProcessState::SB => {
                    let opt = TelnetOption::parse(byte);
                    state = ProcessState::SBData(opt, current + 1);
                }

                // Subnegotiation's data
                ProcessState::SBData(opt, data_start) => {
                    if byte == BYTE_IAC {
                        state = ProcessState::SBDataIAC(opt, data_start);
                    }

                    // XXX: We may need to consider the case that a SB Data
                    // sequence may exceed this buffer
                }

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
                            self.event_queue
                                .push_event(Event::Subnegotiation(opt, data));
                        }
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
                        }
                        // TODO: Write a test case for this
                        b => {
                            self.event_queue.push_event(Event::Error(UnexpectedByte(b)));

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
        let dst = &mut self.process_buffer[dst_start..dst_end];
        dst.copy_from_slice(&self.buffer[data_start..data_end]);
        self.process_buffered_size += data_length;
    }

    fn copy_buffered_data(&mut self, data_start: usize, data_end: usize) -> Box<[u8]> {
        let data = if self.process_buffered_size > 0 {
            // Copy the data to the process buffer
            self.append_data_to_proc_buffer(data_start, data_end);

            let pbe = self.process_buffered_size;
            self.process_buffered_size = 0;

            &self.process_buffer[0..pbe]
        } else {
            &self.buffer[data_start..data_end]
        };

        Box::from(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Error;

    struct MockStream {
        test_data: Vec<u8>,
    }

    impl MockStream {
        fn new(data: Vec<u8>) -> MockStream {
            MockStream { test_data: data }
        }
    }

    impl stream::Stream for MockStream {
        fn set_nonblocking(&self, _nonblocking: bool) -> Result<(), Error> {
            Ok(())
        }

        fn set_read_timeout(&self, _dur: Option<Duration>) -> Result<(), Error> {
            Ok(())
        }
    }

    impl io::Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut offset = 0;
            while offset < buf.len() && offset < self.test_data.len() {
                buf[offset] = self.test_data[offset];
                offset += 1;
            }
            Ok(offset)
        }
    }

    impl io::Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn escapes_double_iac_correctly() {
        let stream = MockStream::new(vec![0x40, 0x5a, 0xff, 0xff, 0x31, 0x34]);

        #[cfg(feature = "zcstream")]
        let stream = ZlibStream::from_stream(stream);

        let stream = Box::new(stream);

        let mut telnet = Telnet::from_stream(stream, 6);

        let expected_bytes_1: [u8; 2] = [0x40, 0x5a];
        let expected_bytes_2: [u8; 3] = [0xff, 0x31, 0x34];

        let event_1 = telnet.read_nonblocking().unwrap();
        if let Event::Data(buffer) = event_1 {
            assert_eq!(buffer.as_ref(), &expected_bytes_1);
        } else {
            panic!();
        }

        let event_2 = telnet.read_nonblocking().unwrap();
        if let Event::Data(buffer) = event_2 {
            assert_eq!(buffer.as_ref(), &expected_bytes_2);
        } else {
            panic!();
        }
    }
}
