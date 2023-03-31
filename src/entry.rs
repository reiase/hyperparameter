use arraystring::CacheString;
use std::ffi::c_void;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Empty,
    Int(i64),
    Float(f64),
    Text(CacheString),
    Boolen(bool),
    UserDefined(*mut c_void),
    PyObject(*mut c_void),
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Value::Text(CacheString::from_str_truncate(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(CacheString::from_str_truncate(value))
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolen(value)
    }
}

impl From<*mut c_void> for Value {
    fn from(value: *mut c_void) -> Self {
        Value::UserDefined(value)
    }
}

#[derive(Clone)]
pub enum EntryValue {
    Single(Value),
    Versioned(Value, Box<EntryValue>),
}

impl EntryValue {
    pub fn get(&self) -> &Value {
        match self {
            EntryValue::Single(val) => val,
            EntryValue::Versioned(val, _) => val,
        }
    }

    pub fn history(&self) -> Option<&Box<EntryValue>> {
        match self {
            EntryValue::Single(_) => None,
            EntryValue::Versioned(_, his) => Some(his),
        }
    }
}

#[derive(Clone)]
pub struct Entry {
    pub key: String,
    pub val: EntryValue,
}

impl Entry {
    pub fn new<T: Into<String>>(key: T, val: Value) -> Entry {
        Entry {
            key: key.into(),
            val: EntryValue::Single(val),
        }
    }

    pub fn get(&self) -> &Value {
        self.val.get()
    }

    pub fn revision<V: Into<Value>>(&mut self, val: V) {
        let value = &self.val;
        self.val = EntryValue::Versioned(val.into(), Box::new(value.clone()));
    }

    pub fn rollback(&mut self) -> Result<(), ()> {
        let his = self.val.history();
        match his {
            None => Err(()),
            Some(h) => {
                self.val = *h.clone();
                Ok(())
            },
        }
    }
}