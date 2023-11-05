use std::io::Read;
use std::io::Write;

use std::net::TcpListener;
use std::net::TcpStream;

pub trait REPL {
    fn feed(&mut self, s: String) -> Option<String>;
    fn is_alive(&self) -> bool;
}

pub struct DebugServer {
    self_addr: Option<String>,
    peer_addr: Option<String>,
    prompt: Option<String>,
}

impl Default for DebugServer {
    fn default() -> Self {
        Self {
            self_addr: Some("127.0.0.1:0".to_string()),
            peer_addr: None,
            prompt: Default::default(),
        }
    }
}

impl DebugServer {
    pub fn new(addr: String) -> Self {
        Self {
            self_addr: Some(addr),
            peer_addr: None,
            prompt: Default::default(),
        }
    }

    pub fn run(&mut self, repl: &mut dyn REPL) {
        let listener = TcpListener::bind(self.self_addr.as_ref().unwrap()).unwrap();
        self.self_addr = match listener.local_addr() {
            Ok(addr) => {
                println!("debug server is started on {}", addr);
                Some(addr.to_string())
            }
            Err(_) => None,
        };
        self.prompt = self.self_addr.as_ref().map(|addr| format!("({})>>", addr));

        for stream in listener.incoming() {
            let exit = stream.map_or(true, |mut s| self.handle(&mut s, repl));
            if exit {
                break;
            };
        }
    }

    fn show_prompt(&self, stream: &mut TcpStream) {
        let _ = stream.write(self.get_prompt().as_bytes());
    }

    fn get_prompt(&self) -> &str {
        self.prompt.as_ref().map_or(">>", |s| s.as_str())
    }

    fn handle(&mut self, stream: &mut TcpStream, repl: &mut dyn REPL) -> bool {
        self.peer_addr = stream.peer_addr().map(|addr| addr.to_string()).ok();
        if let Some(addr) = &self.peer_addr {
            println!("debug server connection from {}", addr);
        }
        self.show_prompt(stream);
        let mut buf = [0; 1024];
        loop {
            let n = match stream.read(&mut buf) {
                Ok(n) if n == 0 => return true,
                Ok(n) => n,
                Err(_) => break,
            };
            let req = String::from_utf8(buf[0..n].to_vec());
            let s = match repl.feed(req.unwrap()) {
                Some(rsp) => format!("{}\n{}", rsp, self.get_prompt()),
                None => self.get_prompt().to_string(),
            };
            if stream.write(s.as_bytes()).is_err() | !repl.is_alive() {
                break;
            }
        }
        !repl.is_alive()
    }
}

pub fn start_debug_server(addr: Option<String>, repl: &mut dyn REPL) {
    let mut server = match addr {
        Some(addr) => DebugServer::new(addr),
        None => DebugServer::default(),
    };
    server.run(repl);
}
