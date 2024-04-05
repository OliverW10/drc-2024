use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread};

use prost::Message;

use crate::{
    logging::Logger, messages
};


// Something to recive commands from
pub trait Commander {
    fn get_latest_message() -> Option<()>;
}

pub struct NetworkComms {
    listener: TcpListener,
    last_recieved: Arc<Mutex<messages::command::DriveCommand>>,
    to_send: Arc<Mutex<Box<messages::diagnostic::FullDiagnostic>>>,
}

const DEFAULT_DRIVE_COMMAND: messages::command::DriveCommand = messages::command::DriveCommand { state: messages::command::CommandType::StateOff.into(), throttle: 0., turn: 0.};

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        let last_recieved = Arc::new(Mutex::new(DEFAULT_DRIVE_COMMAND.clone()));
        let to_send = Arc::new(Mutex::new(Box::new(messages::diagnostic::FullDiagnostic::default())));
        NetworkComms {
            listener: listener,
            last_recieved: last_recieved,
            to_send: to_send,
        }
    }

    // https://beej.us/guide/bgnet/html/index-wide.html
    pub fn start_accept_loop(&self){
        let listener_ref = &self.listener;
        let on_connect = |stream| Self::start_recv_loop(stream, self.last_recieved, self.to_send);
        thread::spawn(||{
            loop {
                let (mut stream, ipAddr) = listener_ref.accept().unwrap();
                on_connect(&mut stream);
            }
        });
    }

    pub fn start_recv_loop(stream: &mut TcpStream, recieved_mutex: Arc<Mutex<messages::command::DriveCommand>>, to_send_mutex: Arc<Mutex<Box<messages::diagnostic::FullDiagnostic>>>) {
        thread::spawn(|| {
            let mut buf: [u8; 2048];
            loop{
                stream.read(&mut buf);
                let recieved = recieved_mutex.lock().unwrap();
                // recieved.

                let to_send = to_send_mutex.lock().unwrap();
                stream.write(&(to_send.encode_length_delimited_to_vec()));
            }
        });
    }
}

impl Logger for NetworkComms {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic){
        let mut to_send = self.to_send.lock().unwrap();

        // TODO: this is very not nice
        to_send.diagnostic = message.diagnostic;
        to_send.map_update = message.map_update;
        to_send.path = message.path;
    }
}

impl Commander for NetworkComms {
    fn get_latest_message() -> Option<()> {Some(())}
}