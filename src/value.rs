use std::{ffi::c_void, mem::replace, sync::Arc};

use phf::phf_map;

use crate::value::VersionedValue::{Single, Versioned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferUnsafe(pub u64, pub unsafe fn(*mut c_void));

impl Drop for DeferUnsafe {
    fn drop(&mut self) {
        unsafe { self.1(self.0 as *mut c_void) }
    }
}

pub type DeferSafe = Arc<DeferUnsafe>;

/// The value type for hyperparameter values
///
/// ```
/// use hyperparameter::value::Value;
/// let v: Value = 1i32.into();
/// println!("{:?}", v);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Empty,
    Int(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    UserDefined(
        u64,               //data
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
        Value::Boolean(value)
    }
}

impl From<*mut c_void> for Value {
    fn from(value: *mut c_void) -> Self {
        Value::UserDefined(value as u64, 0, None)
    }
}

impl Value {
    pub fn managed(ptr: *mut c_void, kind: i32, free: unsafe fn(*mut c_void)) -> Value {
        Value::UserDefined(
            ptr as u64,
            kind,
            Arc::new(DeferUnsafe(ptr as u64, free)).into(),
        )
    }
}

impl TryFrom<&Value> for i64 {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(*v),
            Value::Float(v) => Ok(*v as i64),
            Value::Text(v) => v
                .parse::<i64>()
                .map_err(|_| format!("error convert {} into i64", v)),
            Value::Boolean(v) => Ok(Into::into(*v)),
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and i64".into())
            }
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&Value> for f64 {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(*v as f64),
            Value::Float(v) => Ok(*v),
            Value::Text(v) => v
                .parse::<f64>()
                .map_err(|_| format!("error convert {} into i64", v)),
            Value::Boolean(_) => Err("data type not matched, `Boolean` and i64".into()),
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and f64".into())
            }
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&Value> for String {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(format!("{}", v)),
            Value::Float(v) => Ok(format!("{}", v)),
            Value::Text(v) => Ok(v.clone()),
            Value::Boolean(v) => Ok(format!("{}", v)),
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and str".into())
            }
        }
    }
}

impl TryFrom<Value> for String {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (&value).try_into()
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

impl TryFrom<&Value> for bool {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Empty => Err("empty value error".into()),
            Value::Int(v) => Ok(*v != 0),
            Value::Float(_) => Err("data type not matched, `Float` and bool".into()),
            Value::Text(s) => match STR2BOOL.get(s) {
                Some(v) => Ok(*v),
                None => Err("data type not matched, `Text` and bool".into()),
            },
            Value::Boolean(v) => Ok(*v),
            Value::UserDefined(_, _, _) => {
                Err("data type not matched, `UserDefined` and str".into())
            }
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

#[derive(Debug, Clone)]
pub enum VersionedValue {
    Single(Value),
    Versioned(Value, Box<VersionedValue>),
}

impl VersionedValue {
    pub fn value(&self) -> &Value {
        match self {
            Single(val) => val,
            Versioned(val, _) => val,
        }
    }

    pub fn shallow(&self) -> VersionedValue {
        match self {
            Single(v) => Single(v.clone()),
            Versioned(v, _) => Single(v.clone()),
        }
    }

    pub fn update<V: Into<Value>>(&mut self, val: V) -> Value {
        let val = val.into();
        let old = match self {
            Single(old) => old,
            Versioned(old, _) => old,
        };
        replace(old, val)
    }

    pub fn revision<V: Into<Value>>(&mut self, val: V) {
        *self = Versioned(val.into(), Box::new(self.clone()));
    }

    pub fn rollback(&mut self) -> bool {
        match self {
            Single(_) => {
                *self = Single(EMPTY);
                false
            }
            Versioned(_, his) => {
                *self = *his.clone();
                true
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::ffi::c_void;

    use crate::value::Value;

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
            assert_eq!(y, x);
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
            assert_eq!(y, x);
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
            "UserDefined(43981, 0, None)".to_string()
        );
    }
}

#[cfg(test)]
mod test_versioned_value {
    use crate::value::VersionedValue::Single;

    #[test]
    fn test_versioned_value() {
        let mut val = Single(0.into());
        assert_eq!(format!("{:?}", val), "Single(Int(0))");

        val.update(2.0);
        assert_eq!(format!("{:?}", val), "Single(Float(2.0))");

        val.revision(true);
        assert_eq!(
            format!("{:?}", val),
            "Versioned(Boolean(true), Single(Float(2.0)))"
        );

        val.revision("str");
        assert_eq!(
            format!("{:?}", val),
            "Versioned(Text(\"str\"), Versioned(Boolean(true), Single(Float(2.0))))"
        );

        assert!(val.rollback());
        assert_eq!(
            format!("{:?}", val),
            "Versioned(Boolean(true), Single(Float(2.0)))"
        );

        assert!(val.rollback());
        assert_eq!(format!("{:?}", val), "Single(Float(2.0))");

        assert!(!val.rollback());
        assert_eq!(format!("{:?}", val), "Single(Empty)");
    }

    proptest! {
        #[test]
        fn test_versioned_value_long_history(x in 0i32..100) {
            let mut v = Single(0.into());
            for i in 0..x {
                v.revision(i);
            }
        }
    }
}
