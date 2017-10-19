
use std::collections::LinkedList;

use option::TelnetOption;
use negotiation::NegotiationAction;

#[derive(Debug)]
pub enum TelnetEvent {
    DataReceived(Box<[u8]>),
    UnknownIAC(u8),
    NegotiationReceived(NegotiationAction, TelnetOption),
    NeogitationSent(NegotiationAction, TelnetOption),
    SBReceived(u8, Box<[u8]>), // XXX: Temp
    ItShouldNotBeHere(String), // please contact the developer
    Error(String) // error message
}

pub struct TelnetEventQueue {
    queue: LinkedList<TelnetEvent>
}

impl TelnetEventQueue {
    pub fn new() -> TelnetEventQueue {
        TelnetEventQueue {
            queue: LinkedList::new()
        }
    }

    pub fn push_event(&mut self, event: TelnetEvent) {
        self.queue.push_back(event);
    }

    pub fn take_event(&mut self) -> Option<TelnetEvent> {
        self.queue.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
