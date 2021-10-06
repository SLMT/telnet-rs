#![allow(clippy::must_use_candidate)]

// A macro to reduce the code duplication in the definition of TelnetOption
macro_rules! telnet_options {
    ($($byt:expr => $tno:ident),+) => {
        /// Telnet options
        #[derive(Debug, Clone, Copy)]
        pub enum TelnetOption {
            $($tno,)+
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
    0 => TransmitBinary,
    1 => Echo,
    2 => Reconnection,
    3 => SuppressGoAhead,
    4 => ApproxMessageSizeNeg,
    5 => Status,
    6 => TimingMark,
    7 => RCTE,
    8 => OutLineWidth,
    9 => OutPageSize,
    10 => NAOCRD,
    11 => NAOHTS,
    12 => NAOHTD,
    13 => NAOFFD,
    14 => NAOVTS,
    15 => NAOVTD,
    16 => NAOLFD,
    17 => XASCII,
    18 => Logout,
    19 => ByteMacro,
    20 => DET,
    21 => SUPDUP,
    22 => SUPDUPOutput,
    23 => SNDLOC,
    24 => TTYPE,
    25 => EOR,
    26 => TUID,
    27 => OUTMRK,
    28 => TTYLOC,
    29 => OPT3270Regime,
    30 => X3PAD,
    31 => NAWS,
    32 => TSPEED,
    33 => LFLOW,
    34 => Linemode,
    35 => XDISPLOC,
    36 => Environment,
    37 => Authentication,
    38 => Encryption,
    39 => NewEnvironment,
    70 => MSSP,
    85 => Compress,
    86 => Compress2,
    93 => ZMP,
    255 => EXOPL
);
