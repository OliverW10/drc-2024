use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use prost::Message;

use crate::{logging::Logger, messages};

// Something to recive commands from
pub trait Commander {
    fn get_latest_message(&self) -> Option<messages::command::DriveCommand>;
}

pub struct NetworkComms {
    last_recieved: Arc<Mutex<messages::command::DriveCommand>>,
// TODO: accumulate diagnotic until it gets sent
    to_send: Arc<Mutex<Box<messages::diagnostic::FullDiagnostic>>>,
}

const DEFAULT_DRIVE_COMMAND: messages::command::DriveCommand = messages::command::DriveCommand {
    state: messages::command::CommandType::StateOff as i32,
    throttle: 0.,
    turn: 0.,
};

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let last_recieved = Arc::new(Mutex::new(DEFAULT_DRIVE_COMMAND.clone()));
        let to_send = Arc::new(Mutex::new(Box::new(
            messages::diagnostic::FullDiagnostic::default(),
        )));
        let new_self = NetworkComms {
            last_recieved: last_recieved,
            to_send: to_send,
        };
        new_self.start_accept_loop();
        new_self
    }

    // https://beej.us/guide/bgnet/html/index-wide.html
    pub fn start_accept_loop(&self) {
        let listener = TcpListener::bind("127.0.0.1:3141").unwrap();
        let last_recieved = Arc::clone(&self.last_recieved);
        let to_send = Arc::clone(&self.to_send);
        // Spawn a thread to accept new connections, waiting to accept is blocking
        // only expect to do this once
        thread::spawn(move || {
            for stream in listener.incoming() {
                Self::start_recv_loop(
                    stream.unwrap(),
                    Arc::clone(&last_recieved),
                    Arc::clone(&to_send),
                );
            }
        });
    }

    pub fn start_recv_loop(
        mut stream: TcpStream,
        recieved_mutex: Arc<Mutex<messages::command::DriveCommand>>,
        to_send_mutex: Arc<Mutex<Box<messages::diagnostic::FullDiagnostic>>>,
    ) {
        // Spawn a thread for each connection to recieve messages
        thread::spawn(move || {
            let mut buf = [0; 2048];
            loop {
                stream.read(&mut buf).unwrap();
                let mut recieved = recieved_mutex.lock().unwrap();
                recieved.merge_length_delimited(&buf[..]).unwrap();

                let to_send = to_send_mutex.lock().unwrap();
                stream
                    .write(&(to_send.encode_length_delimited_to_vec()))
                    .unwrap();
            }
        });
    }
}

impl Logger for NetworkComms {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic) {
        let mut to_send = self.to_send.lock().unwrap();
        to_send
            .merge_length_delimited(&message.encode_length_delimited_to_vec()[..])
            .unwrap();
    }
}

impl Commander for NetworkComms {
    fn get_latest_message(&self) -> Option<messages::command::DriveCommand> {
        Some(messages::command::DriveCommand::default())
    }
}
