extern crate mio;
extern crate toml;
extern crate byteorder;

#[macro_use]
extern crate lazy_static;

mod config;

use mio::{EventLoop, EventSet, PollOpt, Handler, Token};
use mio::udp::UdpSocket;
use std::sync::mpsc;
use std::thread;

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::error;

use std::io::prelude::*;
use std::fs::{File, OpenOptions};

const VERSION: u8 = 0x01;
const SERVER_TOKEN: Token = Token(0);


struct Logger {
    file_map: HashMap<u8, File>,
}

impl Logger {
    fn new() -> Self {

        let mut l = Logger{ file_map: HashMap::<u8, File>::new() };
        let c = config::get_config();
        let sec = c.get("log_files").unwrap().as_table().unwrap();

        for (f, s) in sec {
            let k = s.as_integer().unwrap() as u8;
            let f =  OpenOptions::new().truncate(false).append(true).create(true).open(format!("{}/{}", config::ConfigManager::get_config_str("log_server", "log_dir"), f)).unwrap();
            l.file_map.insert(k, f);
            
        }

        l
    }

    fn write(&mut self, cmd: &u8, data: &[u8]) -> Result<(), Box<error::Error>> {
        Ok(try!(self.file_map.get_mut(cmd).unwrap().write_all(data)))
    }
    
}



struct Worker {
    rx: mpsc::Receiver<(u8, String)>,
}

impl Worker {
    fn work(&self, logger: &Arc<Mutex<Logger>>) -> ! {
        loop {
            let (cmd, data) = self.rx.recv().unwrap();
            println!("thread recv:{}", data);
            let mut l = logger.lock().unwrap();
            let _ = l.write(&cmd, &data.into_bytes());
        }
    }
}


struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    worker_count: usize,
    txs: Vec<mpsc::Sender<(u8, String)>>,
}


impl Server {
    fn new(socket: UdpSocket) -> Self {
        let mut s = Server {
            socket: socket,
            buf: vec![0; 2048],
            worker_count: 0,
            txs: Vec::<mpsc::Sender<(u8, String)>>::new(),
        };

        for _ in 0..config::ConfigManager::get_config_num("log_server", "thread_num") {
            let (tx, rx) = mpsc::channel();
            s.txs.push(tx);

            let l = Arc::new(Mutex::new(Logger::new()));
            
            thread::spawn(move || {
                let l = l.clone();
                let w = Worker{ rx: rx };
                w.work(&l);
            });
        }
        
        s
    }

    fn read(&mut self) {
        match self.socket.recv_from(&mut self.buf) {
            Err(_) => {
                println!("Error while recv_from client.");
                return;
            },
            Ok(None) => {
                println!("recv data None.");
                return;
            },
            Ok(Some((len, _))) => {  
                let ver = self.buf[0];
                let cmd = self.buf[1];

                if ver != VERSION {
                    println!("version error!");
                    return;
                }

                let mut buf = Cursor::new(&self.buf[2..6]);
                let _ = buf.read_u32::<BigEndian>().unwrap();   
                                
                let data = format!("{}\n", String::from_utf8(self.buf[6..len].to_vec()).unwrap());
                
                self.txs[self.worker_count].send((cmd, data)).unwrap();
                self.worker_count += 1;
                return;
            },
                
        }
    }
    
}

impl Handler for Server {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, _: &mut EventLoop<Server>, token: Token, events: EventSet) {
        if events.is_readable() {
            match token {
                SERVER_TOKEN => {
                    self.read();
                },
                _ => {
                    println!("unknown socket event!");
                },
            }
        }
    }
}

    
fn main() {

    let addr = format!("0.0.0.0:{}", config::ConfigManager::get_config_num("log_server", "port")).parse().unwrap();
    let server_socket = UdpSocket::v4().unwrap();
    server_socket.bind(&addr).unwrap();

    let mut event_loop = EventLoop::new().unwrap();

    let mut server = Server::new(server_socket);
    event_loop.register(&server.socket,
                        SERVER_TOKEN,
                        EventSet::readable(),
                        PollOpt::edge()).unwrap();
    event_loop.run(&mut server).unwrap();

}

