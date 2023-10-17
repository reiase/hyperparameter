use std::{io::Error, thread, time::Duration};

use ::backtrace::Backtrace;
use hyperparameter::debug::{start_async_server, start_debug_server, REPL};
use pyo3::{prelude::*, types::PyDict};

struct DebugRepl {
    console: Option<Py<PyAny>>,
    buf: String,
    live: bool,
}

impl Default for DebugRepl {
    fn default() -> Self {
        Self {
            console: Some(create_console()),
            buf: "".to_string(),
            live: true,
        }
    }
}

impl DebugRepl {
    pub fn new(console: Option<Py<PyAny>>) -> DebugRepl {
        DebugRepl {
            console,
            buf: "".to_string(),
            live: true,
        }
    }
    fn process(&mut self, cmd: &str) -> Option<String> {
        if cmd.trim() == "exit".to_string() {
            self.live = false
        }
        let ret = self.console.as_ref().map(|console| {
            Python::with_gil(|py| {
                let args = (cmd.to_string().into_py(py),);
                let ret = console.call_method(py, "push", args, None);
                match ret {
                    Ok(obj) => {
                        if obj.is_none(py) {
                            None
                        } else {
                            Some(obj.to_string())
                        }
                    }
                    Err(err) => Some(err.to_string()),
                }
            })
        });
        if let Some(ret) = ret {
            ret
        } else {
            None
        }
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

fn create_console() -> Py<PyAny> {
    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        let ret = Python::run(
            py,
            r#"
import hyperparameter
ret = hyperparameter.DebugConsole()
ret.init()
    "#,
            None,
            Some(locals),
        );
        if ret.is_err() {
            ret.map_err(|err| {
                err.print(py);
            })
            .unwrap();
            py.None();
        }
        let ret = match locals.get_item("ret").unwrap() {
            Some(x) => x.to_object(py),
            None => py.None(),
        };
        ret
    })
}

pub fn debug_callback(addr: Option<String>) {
    let console = create_console();
    let mut repl = DebugRepl::new(Some(console));
    start_debug_server(addr, &mut repl);
}

#[pyfunction]
#[pyo3(signature = (addr=None, background=false))]
pub fn enable_debug_server(addr: Option<String>, background: bool) -> Result<(), Error> {
    unsafe {
        let tmp = addr.clone();
        signal_hook::low_level::register(signal_hook::consts::SIGUSR1, move || {
            debug_callback(tmp.clone())
        })?;
        let tmp = addr.clone();
        signal_hook::low_level::register(signal_hook::consts::SIGABRT, move || {
            debug_callback(tmp.clone())
        })?;
    }
    if background {
        thread::spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(start_async_server::<DebugRepl>(addr))
                .unwrap();
        });
    }
    Ok(())
}

#[pyfunction]
pub fn sleep(secs: u64) {
    std::thread::sleep(Duration::from_secs(secs))
}

#[pyfunction]
pub fn backtrace() -> String {
    let bt = Backtrace::new();
    format!("{:?}", bt)
}
