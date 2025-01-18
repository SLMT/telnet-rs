#![allow(clippy::must_use_candidate)]

// A macro to reduce the code duplication in the definition of TelnetOption
macro_rules! telnet_options {
    ($($(#[doc = $attrs:literal]
         )*$byt:literal => $tno:ident),+) => {
        /// Telnet options
        ///
        /// Options used when negotiating connection settings as defined in
        /// [RFC 1340](https://www.rfc-editor.org/rfc/rfc1340#page-75).
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
        pub enum TelnetOption {
            $($(#[doc = $attrs])*$tno,)+
            UnknownOption(u8),
        }

        impl TelnetOption {
            pub fn parse(byte: u8) -> TelnetOption {
                match byte {
                    $($byt => TelnetOption::$tno,)+
                    byte => TelnetOption::UnknownOption(byte)
                }
            }

            pub fn as_byte(&self) -> u8 {
                match *self {
                    $(TelnetOption::$tno => $byt,)+
                    TelnetOption::UnknownOption(byte) => byte
                }
            }
        }
    }
}

telnet_options!(
    /// With the binary transmission option in effect, the receiver should
    /// interpret characters received from the transmitter which are not
    /// preceded with IAC as 8 bit binary data, with the exception of IAC
    /// followed by IAC which stands for the 8 bit binary data with the
    /// decimal value 255. IAC followed by an effective TELNET command (plus
    /// any additional characters required to complete the command) is still
    /// the command even with the binary transmission option in effect. IAC
    /// followed by a character which is not a defined TELNET command has the
    /// same meaning as IAC followed by NOP, although an IAC followed by an
    /// undefined command should not normally be sent in this mode.
    ///
    /// From [RFC 856](https://www.rfc-editor.org/rfc/rfc856.html)
    0 => TransmitBinary,
    /// When the echoing option is in effect, the party at the end performing
    /// the echoing is expected to transmit (echo) data characters it
    /// receives back to the sender of the data characters.  The option does
    /// not require that the characters echoed be exactly the characters
    /// received (for example, a number of systems echo the ASCII ESC
    /// character with something other than the ESC character).  When the
    /// echoing option is not in effect, the receiver of data characters
    /// should not echo them back to the sender; this, of course, does not
    /// prevent the receiver from responding to data characters received.
    ///
    /// From [RFC 857](https://www.rfc-editor.org/rfc/rfc857.html)
    1 => Echo,
    /// The sender of this command requests the receiver of the command to
    /// be prepared to break the TELNET connection with the sender of the
    /// command and to re-establish the TELNET connection with some other
    /// party (to be specified later).
    ///
    /// From NIC 15391 of 1973
    2 => Reconnection,
    /// When the SUPPRESS-GO-AHEAD option is in effect on the connection
    /// between a sender of data and the receiver of the data, the sender
    /// need not transmit GAs.
    ///
    /// With the SUPPRESS-GO-AHEAD option in effect, the IAC GA command
    /// should be treated as a NOP if received, although IAC GA should not
    /// normally be sent in this mode.
    ///
    /// From [RFC 858](https://www.rfc-editor.org/rfc/rfc858.html)
    3 => SuppressGoAhead,
    /// With the option which specifies the approximate size of messages
    /// transmitted over the connection, the transmitter attempts to send
    /// messages of the specified size unless some other constraint (for
    /// instance, an end of line) requires the message to be sent sooner, or
    /// characters for transmission arrive so fast that the message has to be
    /// bigger than the specified size.  The option is to be used strictly to
    /// improve the STATISTICS (e.g., timing and buffering) of message
    /// reception and transmission -- the option does NOT specify any
    /// absolutes.
    ///
    /// From NIC 15393 of 1973
    4 => ApproxMessageSizeNeg,
    /// This option allows a user/process to verify the current status of
    /// TELNET options (e.g., echoing) as viewed by the person/process on the
    /// other end of the TELNET connection.
    ///
    /// WILL and DO are used only to obtain and grant permission for future
    /// discussion. The actual exchange of status information occurs within
    /// option subcommands (IAC SB STATUS...).
    ///
    /// From [RFC 859](https://www.rfc-editor.org/rfc/rfc859.html)
    5 => Status,
    /// It is sometimes useful for a user or process at one end of a TELNET
    /// connection to be sure that previously transmitted data has been
    /// completely processed, printed, discarded, or otherwise disposed of.
    /// This option provides a mechanism for doing this.  In addition, even
    /// if the option request (DO TIMING-MARK) is refused (by WON'T
    /// TIMING-MARK) the requester is at least assured that the refuser has
    /// received (if not processed) all previous data.
    ///
    /// From [RFC 860](https://www.rfc-editor.org/rfc/rfc860.html)
    6 => TimingMark,
    /// Remote Controlled Transmssion and Echoing
    ///
    /// Alternative echo behaviour intended to reduce user-to-host
    /// communications in high latency environments.
    ///
    /// Defined in  [RFC 726](https://www.rfc-editor.org/rfc/rfc726.html)
    7 => RCTE,
    /// Used to negotiate terminal line width. See [TelnetOption::DET] and
    /// [rfc731](https://www.rfc-editor.org/rfc/rfc731)
    8 => OutLineWidth,
    /// Used to negotiate terminal page height. See [TelnetOption::DET] and
    /// [rfc731](https://www.rfc-editor.org/rfc/rfc731)
    9 => OutPageSize,
    /// Negotiate About Carriage Return Disposition
    ///
    /// Used to determine which party should handle carriage-returns and what
    /// their disposition should be
    ///
    /// From [RFC 652](https://www.rfc-editor.org/rfc/rfc652)
    10 => NAOCRD,
    /// Negotiate About Output Horizontal Tabstops
    ///
    /// Used to determine which party should handle tab stops.
    ///
    /// Defined in [RFC 653](https://www.rfc-editor.org/rfc/rfc653.html)
    11 => NAOHTS,
    /// Negotiate About Output Horizontal Tab Disposition
    ///
    /// Used to determine which party should handle tab stop considerations.
    ///
    /// Defined in [RFC 654](https://www.rfc-editor.org/rfc/rfc654.html)
    12 => NAOHTD,
    /// Negotiate About Output Formfeed Disposition
    ///
    /// Used to determine which party should handle form feeds.
    ///
    /// Defined in [RFC 655](https://www.rfc-editor.org/rfc/rfc655.html)
    13 => NAOFFD,
    /// Negotiate About Output Vertical Tabstops
    ///
    /// Used to determine which party should handle vertical tab stops.
    ///
    /// Defined in [RFC 656](https://www.rfc-editor.org/rfc/rfc656.html)
    14 => NAOVTS,
    /// Negotiate About Output Vertical Tab Disposition
    ///
    /// Used to determine which party should handle veritcal tab stop
    /// considerations.
    ///
    /// Defined in [RFC 657](https://www.rfc-editor.org/rfc/rfc657.html)
    15 => NAOVTD,
    /// Negotiate About Output Linefeed Disposition
    ///
    /// Used to determine which party should handle line feeds.
    ///
    /// Defined in [RFC 655](https://www.rfc-editor.org/rfc/rfc655.html)
    16 => NAOLFD,
    /// Extended ASCII
    ///
    /// This option is to allow the transmission of extended ASCII
    ///
    /// From [RFC 698](https://www.rfc-editor.org/rfc/rfc698.html)
    17 => XASCII,
    /// Request that the user be logged off the server to which it is connected
    ///
    /// Defined in [RFC 727](https://www.rfc-editor.org/rfc/rfc727.html)
    18 => Logout,
    /// Used to define substitute macros that should be expanded by the receiver
    /// into predefined strings.
    ///
    /// Defined in [RFC 735](https://www.rfc-editor.org/rfc/rfc735.html)
    19 => ByteMacro,
    /// Data Entry Terminal
    ///
    /// Use and control an extended 'data entry terminal' allowing direct
    /// addressing of specific locations on the output.
    ///
    /// Defined in [RFC 732](https://www.rfc-editor.org/rfc/rfc732.html) and
    /// [RFC 1043](https://www.rfc-editor.org/rfc/rfc1043.html)
    20 => DET,
    /// Switch to using the SUPDUP protocol instead of the NVT standard terminal.
    ///
    /// If the SUPDUP option is in effect, no further TELNET negotiations are allowed
    ///
    /// Defined in [RFC 736](https://www.rfc-editor.org/rfc/rfc736.html) with
    /// more details about the protocol in
    /// [RFC 734](https://www.rfc-editor.org/rfc/rfc734.html).
    21 => SUPDUP,
    /// Use the SUPDUP protocol for individual messages while in the context of
    /// a telnet session. This is in contrast to the SUPDUP option which
    /// requires all subsequent communications to also use the SUPDUP protocol.
    ///
    /// Defined in [RFC 749](https://www.rfc-editor.org/rfc/rfc749.html)
    22 => SUPDUPOutput,
    /// Send Location
    ///
    /// When the user TELNET program knows the user's location, it should
    /// offer to transmit this information to the server TELNET by sending
    /// IAC WILL SEND-LOCATION.  If the server's system is able to make use
    /// of this information (as can the ITS sites), then the server will
    /// reply with IAC DO SEND-LOCATION.  The user TELNET is then free to
    /// send the location in a subnegotiation at any time.
    ///
    /// From [RFC 779](https://www.rfc-editor.org/rfc/rfc779.html)
    23 => SNDLOC,
    /// Terminal Type
    ///
    /// Used to present a list of available terminal emulation modes to the
    /// server, from which the server can select the one it prefers (for
    /// arbitrary reasons).
    ///
    /// From [RFC 1091](https://www.rfc-editor.org/rfc/rfc1091.html)
    24 => TTYPE,
    /// End of Record
    ///
    /// When the END-OF-RECORD option is in effect on the connection between
    /// a sender of data and the receiver of the data, the sender transmits
    /// EORs.
    ///
    /// From [RFC 885](https://www.rfc-editor.org/rfc/rfc885.html)
    25 => EOR,
    /// TACACS User Identification
    ///
    /// Under TACACS (the TAC Access Control System) a user must be
    /// authenticated (give a correct name/password pair) to a TAC before he
    /// can connect to a host via the TAC. To avoid a second authentication
    /// by the target host, the TAC can pass along the user's proven identity
    /// (his UUID) to the that host. Hosts may accept the TAC's
    /// authentication of the user or not, at their option.
    ///
    /// From [RFC 927](https://www.rfc-editor.org/rfc/rfc927.html)
    26 => TUID,
    /// Output Marking
    ///
    /// Send a banner to a user so that this banner would be displayed on the
    /// workstation screen independently of the application software running
    /// in the server.
    ///
    /// From [RFC 933](https://www.rfc-editor.org/rfc/rfc933.html)
    27 => OUTMRK,
    /// Terminal Location
    ///
    /// Option to send the TTY Location (precursor to IP address) of the
    /// connecting user.
    ///
    /// From [RFC 946](https://www.rfc-editor.org/rfc/rfc946.html)
    28 => TTYLOC,
    /// 3729 Regime
    ///
    /// Allows a telnet server running VM or MVS to negotiate with the telnet
    /// client on the type of data stream (3270 or NVT ASCII) which both sides
    /// are willing to support.
    ///
    /// [RFC 1041](https://www.rfc-editor.org/rfc/rfc1041.html)
    29 => OPT3270Regime,
    /// X.3 Pad
    ///
    /// From [RFC 1053](https://www.rfc-editor.org/rfc/rfc1053.html)
    30 => X3PAD,
    /// Negotiate About Window Size
    ///
    /// Communicate terminal window size from client to server.
    ///
    /// Defined in [RFC 1073](https://www.rfc-editor.org/rfc/rfc1073.html)
    31 => NAWS,
    /// Terminal Speed
    ///
    /// Exchange speed information about attached terminals.
    ///
    /// Defined in [RFC 1079](https://www.rfc-editor.org/rfc/rfc1079.html)
    32 => TSPEED,
    /// Toggle Flow Control
    ///
    /// For remotely toggling flow control between a user telnet process and
    /// the attached terminal. Only flow control of data being transmitted from
    /// the telnet process to the terminal is considered. Many systems will
    /// also allow flow control of data from the terminal to the telnet process,
    /// however there is seldom need to change this behavior repeatedly during
    /// the session.
    ///
    /// From [RFC 1372](https://www.rfc-editor.org/rfc/rfc1372.html)
    33 => LFLOW,
    /// Line Mode
    ///
    /// Linemode Telnet is a way of doing terminal character processing on
    /// the client side of a Telnet connection. While in Linemode with
    /// editing enabled for the local side, network traffic is reduced to a
    /// couple of packets per command line, rather than a couple of packets
    /// per character typed.
    ///
    /// From [RFC 1184](https://www.rfc-editor.org/rfc/rfc1184.html)
    34 => Linemode,
    /// X Display Location
    ///
    /// When a user is running the Telnet client under the X window system,
    /// it is useful for the remote Telnet to know the X display location of
    /// that client. For example, the user might wish to start other X
    /// applications from the remote host using the same display location as
    /// the Telnet client. The purpose of this option is to make this
    /// information available through telnet connections.
    ///
    /// From [RFC 1096](https://www.rfc-editor.org/rfc/rfc1096.html)
    35 => XDISPLOC,
    /// Environment
    ///
    /// Mechanism for passing environment information between a telnet client
    /// and server. Use of this mechanism enables a telnet user to propagate
    /// configuration information to a remote host when connecting.
    ///
    /// From [RFC 1408](https://www.rfc-editor.org/rfc/rfc1408.html)
    36 => Environment,
    /// Authentication
    ///
    /// Option for negotiating an authentication type and mode including whether
    /// encryption should be used and if credentials should be forwarded.
    ///
    /// From [RFC 2941](https://www.rfc-editor.org/rfc/rfc2941.html)
    37 => Authentication,
    /// Encryption
    ///
    /// Option for providing data confidentiality services for the telnet data
    /// stream.
    ///
    /// From [RFC 2946](https://www.rfc-editor.org/rfc/rfc2946.html)
    38 => Encryption,
    /// New Environment
    ///
    /// Mechanism for passing environment information between a telnet client
    /// and server. Use of this mechanism enables a telnet user to propagate
    /// configuration information to a remote host when connecting.
    ///
    /// From [RFC 1572](https://www.rfc-editor.org/rfc/rfc1572.html)
    39 => NewEnvironment,
    70 => MSSP,
    85 => Compress,
    86 => Compress2,
    93 => ZMP,
    /// Extended Options List
    ///
    /// Mechanism to extend the option list beyond the 256 existing options.
    ///
    /// Defined in [RFC 861](https://www.rfc-editor.org/rfc/rfc861.html)
    255 => EXOPL
);
