// This implements the Q method described in Section 7 of RFC 1143

use std::collections::HashMap;

use option::{TelnetOption, TelnetOptionConfig};
use event::{TelnetEvent, TelnetEventQueue};
use net::TelnetStream;
use byte::*;

#[derive(Debug)]
pub enum NegotiationAction {
    Will, Wont, Do, Dont
}

impl NegotiationAction {
    pub fn to_byte(&self) -> u8 {
        match *self {
            NegotiationAction::Will => BYTE_WILL,
            NegotiationAction::Wont => BYTE_WONT,
            NegotiationAction::Do => BYTE_DO,
            NegotiationAction::Dont => BYTE_DONT
        }
    }
}

#[derive(Copy, Clone)]
enum State {
    No,
    Yes,
    WantNoEmpty,
    WantNoOpposite,
    WantYesEmpty,
    WantYesOpposite
}

struct OptState {
    us: State,
    him: State
}

pub struct NegotiationSM {
    map: HashMap<TelnetOption, OptState>
}

impl NegotiationSM {
    pub fn new() -> NegotiationSM {
        NegotiationSM {
            map: HashMap::new()
        }
    }

    pub fn receive_will(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption, opt_config: &TelnetOptionConfig) {
        match self.get_state(false, req_opt) {
            State::No => {
                if opt_config.him {
                    self.set_state(false, req_opt, State::Yes);
                    stream.negotiate(NegotiationAction::Do, req_opt, queue);
                } else {
                    stream.negotiate(NegotiationAction::Dont, req_opt, queue);
                }
            },
            State::Yes => {}, // Ingore
            State::WantNoEmpty => {
                queue.push_event(TelnetEvent::Error(format!("DONT answered by WILL")));
                self.set_state(false, req_opt, State::No);
            },
            State::WantNoOpposite => {
                queue.push_event(TelnetEvent::Error(format!("DONT answered by WILL")));
                self.set_state(false, req_opt, State::Yes);
            },
            State::WantYesEmpty => {
                self.set_state(false, req_opt, State::Yes);
            },
            State::WantYesOpposite => {
                self.set_state(false, req_opt, State::WantNoEmpty);
                stream.negotiate(NegotiationAction::Dont, req_opt, queue);
            },
        }
    }

    pub fn receive_wont(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(false, req_opt) {
            State::No => {}, // Ingore
            State::Yes => {
                self.set_state(false, req_opt, State::No);
                stream.negotiate(NegotiationAction::Dont, req_opt, queue);
            },
            State::WantNoEmpty => {
                self.set_state(false, req_opt, State::No);
            },
            State::WantNoOpposite => {
                self.set_state(false, req_opt, State::WantYesEmpty);
                stream.negotiate(NegotiationAction::Do, req_opt, queue);
            },
            State::WantYesEmpty | State::WantYesOpposite => {
                self.set_state(false, req_opt, State::No);
            },
        }
    }

    fn set_state(&mut self, is_us: bool, opt: TelnetOption,
            new_state: State) {
        let opt_state = self.map.entry(opt).or_insert(OptState {
            us: State::No,
            him: State::No
        });

        if is_us {
            opt_state.us = new_state;
        } else {
            opt_state.him = new_state;
        }
    }

    fn get_state(&self, is_us: bool, opt: TelnetOption) -> State {
        match self.map.get(&opt) {
            Some(s) => {
                if is_us {
                    s.us
                } else {
                    s.him
                }
            },
            None => State::No
        }
    }
}
