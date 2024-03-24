use std::net::TcpListener;

use crate::{
    logging::Logger,
    messages::{
        path::{Path, Points},
        diagnostic::Diagnostic
    }
};


// Something to recive commands from
pub trait Commander {
    fn get_latest_message() -> Option<()>;
}

struct NetworkComms {
    listener: TcpListener
}

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        NetworkComms {
            listener: listener
        }
    }

    pub fn send() {}
}

impl Logger for NetworkComms {
    fn send(&self, path: &Path, new_points: &Points, diagnostic: &Diagnostic) {}
}

impl Commander for NetworkComms {
    fn get_latest_message() -> Option<()> {Some(())}
}