// This implements the Q method described in Section 7 of RFC 1143

use std::collections::HashMap;

use option::TelnetOption;
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
            stream: &TelnetStream, req_opt: TelnetOption, is_remote_allowed: bool) {
        match self.get_state(false, req_opt) {
            State::No => {
                if is_remote_allowed {
                    self.set_state(false, req_opt, State::Yes);
                    stream.send_negotiation(NegotiationAction::Do, req_opt, queue);
                    queue.push_event(TelnetEvent::RemoteEnabled(req_opt));
                } else {
                    stream.send_negotiation(NegotiationAction::Dont, req_opt, queue);
                }
            },
            State::Yes => {}, // Ingore
            State::WantNoEmpty => {
                queue.push_event(TelnetEvent::Error(format!("DONT answered by WILL")));
                self.set_state(false, req_opt, State::No);
                queue.push_event(TelnetEvent::RemoteDisabled(req_opt));
            },
            State::WantNoOpposite => {
                queue.push_event(TelnetEvent::Error(format!("DONT answered by WILL")));
                self.set_state(false, req_opt, State::Yes);
                queue.push_event(TelnetEvent::RemoteEnabled(req_opt));
            },
            State::WantYesEmpty => {
                self.set_state(false, req_opt, State::Yes);
                queue.push_event(TelnetEvent::RemoteEnabled(req_opt));
            },
            State::WantYesOpposite => {
                self.set_state(false, req_opt, State::WantNoEmpty);
                stream.send_negotiation(NegotiationAction::Dont, req_opt, queue);
            }
        }
    }

    pub fn receive_wont(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(false, req_opt) {
            State::No => {}, // Ingore
            State::Yes => {
                self.set_state(false, req_opt, State::No);
                stream.send_negotiation(NegotiationAction::Dont, req_opt, queue);
                queue.push_event(TelnetEvent::RemoteDisabled(req_opt));
            },
            State::WantNoEmpty => {
                self.set_state(false, req_opt, State::No);
                queue.push_event(TelnetEvent::RemoteDisabled(req_opt));
            },
            State::WantNoOpposite => {
                self.set_state(false, req_opt, State::WantYesEmpty);
                stream.send_negotiation(NegotiationAction::Do, req_opt, queue);
            },
            State::WantYesEmpty | State::WantYesOpposite => {
                self.set_state(false, req_opt, State::No);
                queue.push_event(TelnetEvent::RemoteDisabled(req_opt));
            }
        }
    }

    pub fn receive_do(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption, is_local_supported: bool) {
        match self.get_state(true, req_opt) {
            State::No => {
                if is_local_supported {
                    self.set_state(true, req_opt, State::Yes);
                    stream.send_negotiation(NegotiationAction::Will, req_opt, queue);
                    queue.push_event(TelnetEvent::LocalShouldEnable(req_opt));
                } else {
                    stream.send_negotiation(NegotiationAction::Wont, req_opt, queue);
                }
            },
            State::Yes => {}, // Ingore
            State::WantNoEmpty => {
                queue.push_event(TelnetEvent::Error(format!("WONT answered by DO")));
                self.set_state(true, req_opt, State::No);
                queue.push_event(TelnetEvent::LocalShouldDisable(req_opt));
            },
            State::WantNoOpposite => {
                queue.push_event(TelnetEvent::Error(format!("WONT answered by DO")));
                self.set_state(true, req_opt, State::Yes);
                queue.push_event(TelnetEvent::LocalShouldEnable(req_opt));
            },
            State::WantYesEmpty => {
                self.set_state(true, req_opt, State::Yes);
                queue.push_event(TelnetEvent::LocalShouldEnable(req_opt));
            },
            State::WantYesOpposite => {
                self.set_state(true, req_opt, State::WantNoEmpty);
                stream.send_negotiation(NegotiationAction::Wont, req_opt, queue);
            }
        }
    }

    pub fn receive_dont(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(true, req_opt) {
            State::No => {}, // Ingore
            State::Yes => {
                self.set_state(true, req_opt, State::No);
                stream.send_negotiation(NegotiationAction::Wont, req_opt, queue);
                queue.push_event(TelnetEvent::LocalShouldDisable(req_opt));
            },
            State::WantNoEmpty => {
                self.set_state(true, req_opt, State::No);
                queue.push_event(TelnetEvent::LocalShouldDisable(req_opt));
            },
            State::WantNoOpposite => {
                self.set_state(true, req_opt, State::WantYesEmpty);
                stream.send_negotiation(NegotiationAction::Will, req_opt, queue);
            },
            State::WantYesEmpty | State::WantYesOpposite => {
                self.set_state(true, req_opt, State::No);
                queue.push_event(TelnetEvent::LocalShouldDisable(req_opt));
            }
        }
    }

    pub fn inform_enabled(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(true, req_opt) {
            State::No => {
                self.set_state(true, req_opt, State::WantYesEmpty);
                stream.send_negotiation(NegotiationAction::Will, req_opt, queue);
            },
            State::Yes => {
                queue.push_event(TelnetEvent::Error(
                    format!("Option: {:?} is already enabled", req_opt)));
            },
            State::WantNoEmpty => {
                self.set_state(true, req_opt, State::WantNoOpposite);
            },
            State::WantNoOpposite => {
                queue.push_event(TelnetEvent::Error(format!("Already queued an enable request")));
            },
            State::WantYesEmpty => {
                queue.push_event(TelnetEvent::Error(format!("Already negotiating for enable")));
            },
            State::WantYesOpposite => {
                self.set_state(true, req_opt, State::WantYesEmpty);
            }
        }
    }

    pub fn inform_disable(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(true, req_opt) {
            State::No => {
                queue.push_event(TelnetEvent::Error(
                    format!("Option: {:?} is already disabled", req_opt)));
            },
            State::Yes => {
                self.set_state(true, req_opt, State::WantNoEmpty);
                stream.send_negotiation(NegotiationAction::Wont, req_opt, queue);
            },
            State::WantNoEmpty => {
                queue.push_event(TelnetEvent::Error(format!("Already negotiating for disable")));
            },
            State::WantNoOpposite => {
                self.set_state(true, req_opt, State::WantNoEmpty);
            },
            State::WantYesEmpty => {
                self.set_state(true, req_opt, State::WantYesOpposite);
            },
            State::WantYesOpposite => {
                queue.push_event(TelnetEvent::Error(format!("Already queued an disable request")));
            }
        }
    }

    pub fn ask_to_enable(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(false, req_opt) {
            State::No => {
                self.set_state(false, req_opt, State::WantYesEmpty);
                stream.send_negotiation(NegotiationAction::Do, req_opt, queue);
            },
            State::Yes => {
                queue.push_event(TelnetEvent::Error(
                    format!("Option: {:?} is already enabled", req_opt)));
            },
            State::WantNoEmpty => {
                self.set_state(false, req_opt, State::WantNoOpposite);
            },
            State::WantNoOpposite => {
                queue.push_event(TelnetEvent::Error(format!("Already queued an enable request")));
            },
            State::WantYesEmpty => {
                queue.push_event(TelnetEvent::Error(format!("Already negotiating for enable")));
            },
            State::WantYesOpposite => {
                self.set_state(false, req_opt, State::WantYesEmpty);
            }
        }
    }

    pub fn ask_to_disable(&mut self, queue: &mut TelnetEventQueue,
            stream: &TelnetStream, req_opt: TelnetOption) {
        match self.get_state(false, req_opt) {
            State::No => {
                queue.push_event(TelnetEvent::Error(
                    format!("Option: {:?} is already disabled", req_opt)));
            },
            State::Yes => {
                self.set_state(false, req_opt, State::WantNoEmpty);
                stream.send_negotiation(NegotiationAction::Dont, req_opt, queue);
            },
            State::WantNoEmpty => {
                queue.push_event(TelnetEvent::Error(format!("Already negotiating for disable")));
            },
            State::WantNoOpposite => {
                self.set_state(false, req_opt, State::WantNoEmpty);
            },
            State::WantYesEmpty => {
                self.set_state(false, req_opt, State::WantYesOpposite);
            },
            State::WantYesOpposite => {
                queue.push_event(TelnetEvent::Error(format!("Already queued an disable request")));
            }
        }
    }

    // TODO: States check for users

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
