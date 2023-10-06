use phf::phf_map;
use std::{ffi::c_void, sync::Arc};

#[derive(Debug, Clone, PartialEq)]
pub struct DeferUnsafe(pub *mut c_void, pub unsafe fn(*mut c_void));
impl Drop for DeferUnsafe {
    fn drop(&mut self) {
        unsafe { self.1(self.0) }
    }
}

pub type DeferSafe = Arc<DeferUnsafe>;

/// The value type for hyperparameter values
///
/// ```
/// use hyperparameter::entry::Value;
/// let v: Value = 1i32.into();
/// println!("{:?}", v);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Empty,
    Int(i64),
    Float(f64),
    Text(String),
    Boolen(bool),
    UserDefined(
        *mut c_void,       //data
        i32,               //kind
        Option<DeferSafe>, // de-allocator
    ),
}

pub const EMPTY: Value = Value::Empty;

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(x) => {
                let y: Value = x.into();
                y
            }
            None => Value::Empty,
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value.into())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Value::Text(value.clone())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolen(value)
    }
}

impl From<*mut c_void> for Value {
    fn from(value: *mut c_void) -> Self {
        Value::UserDefined(value, 0, None)
    }
}

impl Value {
    pub fn managed(ptr: *mut c_void, kind: i32, free: unsafe fn(*mut c_void)) -> Value {
        Value::UserDefined(ptr, kind, Arc::new(DeferUnsafe(ptr, free)).into())
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
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and i64".into())
            }
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
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and f64".into())
            }
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
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and str".into())
            }
        }
    }
}

static STR2BOOL: phf::Map<&'static str, bool> = phf_map! {
    "true" => true,
    "True" => true,
    "TRUE" => true,
    "T" => true,
    "yes" => true,
    "y" => true,
    "Yes" => true,
    "YES" => true,
    "Y" => true,
    "on" => true,
    "On" => true,
    "ON" => true,

    "false" => false,
    "False" => false,
    "FALSE" => false,
    "F" => false,
    "no" => false,
    "n" => false,
    "No" => false,
    "NO" => false,
    "N" => false,
    "off" => false,
    "Off" => false,
    "OFF" => false,
};

impl TryFrom<Value> for bool {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(v != 0),
            Value::Float(_) => Err("data type not matched, `Float` and bool".into()),
            Value::Text(s) => match STR2BOOL.get(&s) {
                Some(v) => Ok(v.clone()),
                None => Err("data type not matched, `Text` and bool".into()),
            },
            Value::Boolen(v) => Ok(v),
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and str".into())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum VersionedValue {
    Single(Value),
    Versioned(Value, Box<VersionedValue>),
}

impl VersionedValue {
    pub fn get(&self) -> &Value {
        match self {
            VersionedValue::Single(val) => val,
            VersionedValue::Versioned(val, _) => val,
        }
    }

    pub fn history(&self) -> Option<&Box<VersionedValue>> {
        match self {
            VersionedValue::Single(_) => None,
            VersionedValue::Versioned(_, his) => Some(his),
        }
    }

    pub fn shallow_copy(&self) -> VersionedValue {
        match self {
            VersionedValue::Single(v) => Self::Single(v.clone()),
            VersionedValue::Versioned(v, _) => Self::Single(v.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub key: String,
    pub val: VersionedValue,
}

impl Entry {
    pub fn new<T: Into<String>, V: Into<Value>>(key: T, val: V) -> Entry {
        Entry {
            key: key.into(),
            val: VersionedValue::Single(val.into()),
        }
    }

    pub fn get(&self) -> &Value {
        self.val.get()
    }

    pub fn clone_value(&self) -> Value {
        self.val.get().clone()
    }

    pub fn shallow_copy(&self) -> Entry {
        Entry { key: self.key.clone(), val: self.val.shallow_copy() }
    }

    pub fn update<V: Into<Value>>(&mut self, val: V) {
        if let Some(his) = self.val.history() {
            self.val = VersionedValue::Versioned(val.into(), his.clone());
        } else {
            self.val = VersionedValue::Single(val.into());
        }
    }

    pub fn revision<V: Into<Value>>(&mut self, val: V) {
        let value = &self.val;
        self.val = VersionedValue::Versioned(val.into(), Box::new(value.clone()));
    }

    pub fn rollback(&mut self) -> Result<(), ()> {
        if let Some(h) = self.val.history() {
            self.val = *h.clone();
            return Ok(());
        }
        self.val = VersionedValue::Single(EMPTY);
        return Err(());
    }
}

#[cfg(test)]
mod test {
    use std::ffi::c_void;

    use crate::entry::Value;
    proptest! {
        #[test]
        fn create_int_value_from_i32(x in 0i32..100) {
            let y: Value = x.into();
            let y: i64 = y.try_into().unwrap();
            assert_eq!(y, x as i64);
        }

        #[test]
        fn create_int_value_from_i64(x in 0i64..100) {
            let y: Value = x.into();
            let y: i64 = y.try_into().unwrap();
            assert_eq!(y, x as i64);
        }

        #[test]
        fn create_float_value_from_f32(x in 0f32..100.0) {
            let y: Value = x.into();
            let y: f64 = y.try_into().unwrap();
            assert_eq!(y, x as f64);
        }

        #[test]
        fn create_float_value_from_f64(x in 0f64..100.0) {
            let y: Value = x.into();
            let y: f64 = y.try_into().unwrap();
            assert_eq!(y, x as f64);
        }

        #[test]
        fn int_value_into_string(x in 0i32..100) {
            let y: Value = x.into();
            let y: String = y.try_into().unwrap();
            assert_eq!(y, format!("{}", x));
        }

        #[test]
        fn float_value_into_string(x in 0f64..100.0) {
            let y: Value = x.into();
            let y: String = y.try_into().unwrap();
            assert_eq!(y, format!("{}", x));
        }

        #[test]
        fn bool_value_into_string(x: bool) {
            let y: Value = x.into();
            let y: String = y.try_into().unwrap();
            assert_eq!(y, format!("{}", x));
        }
    }

    #[test]
    fn test_user_defined_value() {
        let ptr: *mut c_void = 0x00abcd as *mut c_void;
        let ptr: Value = ptr.into();
        assert_eq!(
            format!("{:?}", ptr),
            "UserDefined(0xabcd, 0, None)".to_string()
        );
    }
}

#[cfg(test)]
mod test_versioned_value {
    use super::Entry;

    #[test]
    fn test_versioned_value() {
        let mut v = Entry::new("0", 0);
        assert_eq!(format!("{:?}", v.val), "Single(Int(0))");

        v.revision(1.0);
        assert_eq!(
            format!("{:?}", v.val),
            "Versioned(Float(1.0), Single(Int(0)))"
        );

        v.update("2.0");
        assert_eq!(
            format!("{:?}", v.val),
            "Versioned(Text(\"2.0\"), Single(Int(0)))"
        );

        let _ = v.rollback();
        assert_eq!(format!("{:?}", v.val), "Single(Int(0))");

        let check = v.rollback();
        assert_eq!(format!("{:?}", v.val), "Single(Empty)");
        assert!(check.is_err());
    }

    proptest! {
        #[test]
        fn test_versioned_value_long_history(x in 0i32..100) {
            let mut v = Entry::new("0", 0);
            for i in 0..x {
                v.revision(i);
            }
        }
    }
}
