
use std::collections::LinkedList;

use option::TelnetOption;
use negotiation::NegotiationAction;

#[derive(Debug)]
pub enum TelnetEvent {
    Data(Box<[u8]>),
    UnknownIAC(u8),
    Negotiation(NegotiationAction, TelnetOption),
    Subnegotiation(TelnetOption, Box<[u8]>),

    // Others
    TimedOut, // When timeout happens
    NoData, // only happens in non-blocking

    // Error
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
