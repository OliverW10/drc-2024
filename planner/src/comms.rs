use std::{io::Read, net::{TcpListener, TcpStream}, thread};

use crate::{
    logging::Logger,
    messages::{
        path::{Path, MapUpdate},
        diagnostic::Diagnostic
    }
};


// Something to recive commands from
pub trait Commander {
    fn get_latest_message() -> Option<()>;
}

pub struct NetworkComms {
    listener: TcpListener
}

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        NetworkComms {
            listener: listener
        }
    }
    
    pub fn start_accept_loop(&self){
        // thread::spawn(||{
        //     loop {
        //         let (stream, ipAddr) = self.listener.accept().unwrap();
        //         self.start_recv_loop(stream);
        //     }
        // });
    }

    pub fn start_recv_loop(&self, stream: TcpStream) {
        // thread::spawn(|| {
        //     let buf: [u8; 2048];
        //     loop{
        //         stream.read(&mut buf, );
        //     }
        // });
    }
}

impl Logger for NetworkComms {
    fn send_messages(&mut self, path: &Path, new_points: &MapUpdate, diagnostic: &Diagnostic) {
    }
}

impl Commander for NetworkComms {
    fn get_latest_message() -> Option<()> {Some(())}
}