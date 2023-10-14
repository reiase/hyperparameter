use std::io::Error;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use signal_hook;

use hyperparameter::debug_server::start_debug_server;
use hyperparameter::debug_server::REPL;

struct DebugRepl {
    buf: String,
    live: bool,
}

impl DebugRepl {
    pub fn new() -> DebugRepl {
        DebugRepl {
            buf: "".to_string(),
            live: true,
        }
    }
    fn process(&mut self, cmd: &String) -> Option<String> {
        if cmd.trim() == "exit".to_string() {
            self.live = false
        }
        return Some(cmd.clone());
    }
}

impl REPL for DebugRepl {
    fn feed(&mut self, s: String) -> Option<String> {
        self.buf += &s;
        if !self.buf.contains("\n") {
            return None;
        }
        let cmd = match self.buf.split_once("\n") {
            Some((cmd, rest)) => {
                let cmd = cmd.to_string();
                self.buf = rest.to_string();
                Some(cmd)
            }
            None => None,
        };
        if let Some(cmd) = cmd {
            self.process(&cmd)
        } else {
            None
        }
    }

    fn is_alive(&self) -> bool {
        self.live
    }
}

fn debug_callback() {
    let mut repl = DebugRepl::new();
    start_debug_server(Some("127.0.0.1:9900".to_string()), &mut repl);
    exit(0)
}

pub fn main() -> Result<(), Error> {
    unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGUSR1, move || debug_callback())?;
        signal_hook::low_level::register(signal_hook::consts::SIGABRT, move || debug_callback())?;
    }

    for i in 0..1000 {
        sleep(Duration::from_secs(1));
        println!("{}", i);
    }

    Ok(())
}
