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
    to_send: Arc<Mutex<messages::diagnostic::FullDiagnostic>>,
}

const DEFAULT_DRIVE_COMMAND: messages::command::DriveCommand = messages::command::DriveCommand {
    state: messages::command::CommandMode::StateOff as i32,
    throttle: 0.,
    turn: 0.,
};

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let last_recieved = Arc::new(Mutex::new(DEFAULT_DRIVE_COMMAND.clone()));
        let to_send = Arc::new(Mutex::new(
            messages::diagnostic::FullDiagnostic::default(),
        ));
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
        to_send_mutex: Arc<Mutex<messages::diagnostic::FullDiagnostic>>,
    ) {
        // Spawn a thread for each connection to recieve messages
        thread::spawn(move || {
            let mut buf = [0; 2048];
            loop {
                let bytes_read = stream.read(&mut buf).unwrap(); // i think this will panic too if the connection is closed
                let mut recieved = recieved_mutex.lock().unwrap();
                recieved.merge_length_delimited(&buf[..]).unwrap();
                println!("recived {:?}", recieved);
                let mut to_send = to_send_mutex.lock().unwrap();
                let to_send_buf = to_send.encode_length_delimited_to_vec();
                let sent = stream.write(&to_send_buf);
                if let Err(err) = sent {
                    println!(
                        "Could respond to client: '{}', closing this recieving thread",
                        err
                    );
                    break;
                }

                reset_diagnostic(&mut to_send);

                println!(
                    "Recieved {} bytes, sending {} bytes",
                    bytes_read,
                    to_send_buf.len()
                );
            }
        });
    }
}

impl Logger for NetworkComms {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic) {
        let mut to_send = self.to_send.lock().unwrap();
        accumulate_diagnostic(&mut to_send, message);
    }
}

fn accumulate_diagnostic(target: &mut messages::diagnostic::FullDiagnostic, new: &messages::diagnostic::FullDiagnostic) {
    // Replace the diagnostic and path with most recent
    target.diagnostic = new.diagnostic.clone();
    target.path = new.path.clone();
    // Accumulate map updates
    target.map_update = match (&mut target.map_update, &new.map_update) {
        (None, _) => new.map_update.clone(), 
        (Some(old_map), Some(new_map))  => {
            old_map.points_added.append(&mut new_map.points_added.clone());
            old_map.removed_ids.append(&mut new_map.removed_ids.clone());
            target.map_update.clone() // TODO: avoid clone
        },
        (Some(_), None) => target.map_update.clone(),
    };
}

fn reset_diagnostic(target: &mut messages::diagnostic::FullDiagnostic) {
    target.map_update = None;
}

impl Commander for NetworkComms {
    fn get_latest_message(&self) -> Option<messages::command::DriveCommand> {
        Some(messages::command::DriveCommand::default())
    }
}
