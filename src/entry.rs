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
    PyObject(*mut c_void, unsafe fn(*mut c_void)),
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

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(CacheString::from_str_truncate(value))
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

impl TryFrom<Value> for i64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(v),
            Value::Float(v) => Ok(v as i64),
            Value::Text(v) => v
                .parse::<i64>()
                .or_else(|_| Err(format!("error convert {} into i64", v))),
            Value::Boolen(v) => Ok(v.into()),
            Value::UserDefined(_) => Err("data type not matched, `Userdefined` and i64".into()),
            Value::PyObject(_, _) => Err("data type not matched, `PyObject` and i64".into()),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(v as f64),
            Value::Float(v) => Ok(v),
            Value::Text(v) => v
                .parse::<f64>()
                .or_else(|_| Err(format!("error convert {} into i64", v))),
            Value::Boolen(_) => Err("data type not matched, `Boolen` and i64".into()),
            Value::UserDefined(_) => Err("data type not matched, `Userdefined` and f64".into()),
            Value::PyObject(_, _) => Err("data type not matched, `PyObject` and f64".into()),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(format!("{}", v)),
            Value::Float(v) => Ok(format!("{}", v)),
            Value::Text(v) => Ok(v.to_string()),
            Value::Boolen(v) => Ok(format!("{}", v)),
            Value::UserDefined(_) => Err("data type not matched, `Userdefined` and str".into()),
            Value::PyObject(_, _) => Err("data type not matched, `PyObject` and str".into()),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(v != 0),
            Value::Float(_) => Err("data type not matched, `Float` and bool".into()),
            Value::Text(_) => Err("data type not matched, `Text` and bool".into()),
            Value::Boolen(v) => Ok(v),
            Value::UserDefined(_) => Err("data type not matched, `Userdefined` and str".into()),
            Value::PyObject(_, _) => Err("data type not matched, `PyObject` and str".into()),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

    pub fn clone_value(&self) -> Value {
        self.val.get().clone()
    }

    pub fn update<V: Into<Value>>(&mut self, val: V) {
        let his = self.val.history();
        if his.is_none() {
            self.val = EntryValue::Single(val.into());
        } else {
            self.val = EntryValue::Versioned(val.into(), (*his.unwrap()).clone());
        }
    }

    pub fn revision<V: Into<Value>>(&mut self, val: V) {
        let value = &self.val;
        self.val = EntryValue::Versioned(val.into(), Box::new(value.clone()));
    }

    pub fn rollback(&mut self) -> Result<(), ()> {
        let val = self.val.get();
        let his = self.val.history();
        match his {
            None => Err(()),
            Some(h) => {
                match val {
                    Value::PyObject(obj, free) => unsafe {
                        free(*obj);
                    },
                    _ => {}
                }
                self.val = *h.clone();
                Ok(())
            }
        }
    }
}
