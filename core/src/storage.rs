use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::value::Value;
use crate::value::VersionedValue;
use crate::value::EMPTY;
use crate::xxh::XXHashable;

#[derive(Debug, Clone)]
pub struct Entry {
    pub key: String,
    pub val: VersionedValue,
}

impl Entry {
    pub fn new<T: Into<String>, V: Into<Value>>(key: T, val: V) -> Entry {
        Entry {
            key: key.into(),
            val: VersionedValue::from(val.into()),
        }
    }

    pub fn value(&self) -> &Value {
        self.val.value()
    }

    pub fn clone_value(&self) -> Value {
        self.val.value().clone()
    }

    pub fn shallow(&self) -> Entry {
        Entry {
            key: self.key.clone(),
            val: self.val.shallow(),
        }
    }
}

pub type Params = BTreeMap<u64, Entry>;

pub trait MultipleVersion<K> {
    fn update<V: Into<Value>>(&mut self, key: K, val: V);
    fn revision<V: Into<Value>>(&mut self, key: K, val: V);
    fn rollback(&mut self, key: K);
}

impl MultipleVersion<u64> for Params {
    fn update<V: Into<Value>>(&mut self, key: u64, val: V) {
        if let Some(e) = self.get_mut(&key) {
            e.val.update(val);
        }
    }

    fn revision<V: Into<Value>>(&mut self, key: u64, val: V) {
        if let Some(e) = self.get_mut(&key) {
            e.val.revision(val);
        }
    }

    fn rollback(&mut self, key: u64) {
        if let Some(e) = self.get_mut(&key) {
            if !e.val.rollback() {
                self.remove(&key);
            }
        }
    }
}

thread_local! {
    pub static THREAD_STORAGE: RefCell<Storage> = create_thread_storage();
}

fn create_thread_storage() -> RefCell<Storage> {
    let ts = RefCell::new(Storage::default());
    // Use read lock for concurrent access during thread initialization
    if let Ok(global_storage) = GLOBAL_STORAGE.read() {
        ts.borrow_mut().params.clone_from(&global_storage.params);
    }
    // If lock is poisoned, continue with empty storage
    ts
}

lazy_static! {
    /// Global storage shared across all threads.
    /// Uses RwLock to allow concurrent reads while maintaining exclusive writes.
    static ref GLOBAL_STORAGE: RwLock<Storage> = RwLock::new(Storage::default());
}

/// Freezes the current thread's storage into the global storage.
///
/// This function copies all parameters from the current thread's storage
/// into the global storage, making them available to newly created threads.
///
/// # Performance
/// Uses a write lock, which will block other threads from reading or writing
/// the global storage until this operation completes.
pub fn frozen_global_storage() {
    THREAD_STORAGE.with(|ts| {
        let thread_params = ts.borrow().params.clone();
        // Use write lock for exclusive access during update
        if let Ok(mut global_storage) = GLOBAL_STORAGE.write() {
            global_storage.params = thread_params;
        }
        // If lock is poisoned, silently fail (other threads may have panicked)
    });
}

#[derive(Debug)]
pub struct Storage {
    pub params: Params,
    pub history: Vec<HashSet<u64>>,
}

unsafe impl Send for Storage {}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            params: Params::new(),
            history: vec![HashSet::new()],
        }
    }
}

impl Storage {
    pub fn enter(&mut self) {
        self.history.push(HashSet::new());
    }

    pub fn exit(&mut self) -> Params {
        let mut changes = Params::new();
        let history_level = self.history.pop().expect("Storage::exit() called but history stack is empty. This indicates a mismatch between enter() and exit() calls.");
        for key in history_level {
            let entry = self.params.get(&key).expect("Storage::exit() found key in history but not in params. This indicates corrupted storage state.");
            changes.insert(key, entry.shallow());
            self.params.rollback(key);
        }
        changes
    }

    pub fn get_entry(&self, key: u64) -> Option<&Entry> {
        self.params.get(&key)
    }

    pub fn put_entry(&mut self, key: u64, entry: Entry) -> Option<Entry> {
        self.params.insert(key, entry)
    }

    pub fn del_entry(&mut self, key: u64) {
        self.params.remove(&key);
    }

    pub fn get<T: XXHashable>(&self, key: T) -> &Value {
        let hkey = key.xxh();
        if let Some(e) = self.params.get(&hkey) {
            e.value()
        } else {
            &EMPTY
        }
    }

    pub fn put<T: Into<String> + XXHashable, V: Into<Value> + Clone>(&mut self, key: T, val: V) {
        let hkey = key.xxh();
        let key: String = key.into();
        let current_history = self.history.last_mut().expect(
            "Storage::put() called but history stack is empty. Storage should always have at least one history level (created in Default)."
        );
        if current_history.contains(&hkey) {
            self.params.update(hkey, val);
        } else {
            if let std::collections::btree_map::Entry::Vacant(e) = self.params.entry(hkey) {
                e.insert(Entry {
                    key,
                    val: VersionedValue::from(val.into()),
                });
            } else {
                self.params.revision(hkey, val);
            }
            current_history.insert(hkey);
        }
    }

    pub fn del<T: XXHashable>(&mut self, key: T) {
        let hkey = key.xxh();
        let current_history = self.history.last_mut().expect(
            "Storage::del() called but history stack is empty. Storage should always have at least one history level (created in Default)."
        );
        if current_history.contains(&hkey) {
            self.params.update(hkey, None::<i32>);
        } else {
            self.params.revision(hkey, None::<i32>);
            current_history.insert(hkey);
        }
    }

    pub fn keys(&self) -> Vec<String> {
        self.params
            .values()
            .filter(|x| !matches!(x.value(), Value::Empty))
            .map(|x| x.key.clone())
            .collect()
    }
}

// Hashable trait is kept for potential future use
#[allow(dead_code)]
pub trait Hashable {}

#[allow(dead_code)]
impl Hashable for String {}

#[allow(dead_code)]
impl Hashable for &String {}

#[allow(dead_code)]
impl Hashable for &str {}

#[allow(dead_code)]
impl Hashable for str {}

pub trait GetOrElse<K, T> {
    fn get_or_else(&self, key: K, dval: T) -> T;
}

impl<T> GetOrElse<u64, T> for Storage
where
    T: Into<Value> + TryFrom<Value> + for<'a> TryFrom<&'a Value>,
{
    fn get_or_else(&self, key: u64, dval: T) -> T {
        if let Some(val) = self.params.get(&key) {
            match val.value().try_into() {
                Ok(v) => v,
                Err(_) => dval,
            }
        } else {
            dval
        }
    }
}

impl<K, T> GetOrElse<K, T> for Storage
where
    K: Into<String> + XXHashable,
    T: Into<Value> + TryFrom<Value> + for<'a> TryFrom<&'a Value>,
{
    fn get_or_else(&self, key: K, dval: T) -> T {
        let hkey = key.xxh();
        self.get_or_else(hkey, dval)
    }
}

#[cfg(test)]
mod tests {
    use super::GetOrElse;
    use super::Storage;

    #[test]
    fn test_storage_create() {
        let _ = Storage::default();
    }

    #[test]
    fn test_storage_put_get() {
        let mut s = Storage::default();
        s.put("1", 1);
        s.put("2.0", 2.0);
        s.put("str", "str");
        s.put("bool", true);

        let v: i64 = s
            .get("1")
            .clone()
            .try_into()
            .expect("Failed to convert '1' to i64");
        assert_eq!(1, v);

        let v: f64 = s
            .get("2.0")
            .clone()
            .try_into()
            .expect("Failed to convert '2.0' to f64");
        assert_eq!(2.0, v);

        let v: String = s
            .get("str")
            .clone()
            .try_into()
            .expect("Failed to convert 'str' to String");
        assert_eq!("str", v);
    }

    #[test]
    fn test_storage_get_or_else() {
        let mut s = Storage::default();
        s.put("1", 1);
        s.put("2.0", 2.0);
        s.put("str", "str");
        s.put("bool", true);

        assert_eq!(1, s.get_or_else("1", 0));
        assert_eq!(2.0, s.get_or_else("2.0", 0.0));
        assert_eq!("str", s.get_or_else("str".to_string(), "".to_string()));
        assert_eq!(true, s.get_or_else("bool", false));
    }

    #[test]
    fn test_storage_enter_exit() {
        let mut s0 = Storage::default();
        s0.put("a", 1);
        s0.put("b", 2.0);
        s0.enter();

        // check parameter "a" and "b"
        let v: i64 = s0
            .get("a")
            .clone()
            .try_into()
            .expect("Failed to convert 'a' to i64");
        assert_eq!(1, v);

        let v: f64 = s0
            .get("b")
            .clone()
            .try_into()
            .expect("Failed to convert 'b' to f64");
        assert_eq!(2.0, v);

        s0.put("a", 2);
        s0.put("b", 3.0);
        let v: i64 = s0
            .get("a")
            .clone()
            .try_into()
            .expect("Failed to convert 'a' to i64 after update");
        assert_eq!(2, v);

        let v: f64 = s0
            .get("b")
            .clone()
            .try_into()
            .expect("Failed to convert 'b' to f64 after update");
        assert_eq!(3.0, v);

        s0.exit();
        // check parameter "a" and "b"
        let v: i64 = s0
            .get("a")
            .clone()
            .try_into()
            .expect("Failed to convert 'a' to i64 after exit");
        assert_eq!(1, v);

        let v: f64 = s0
            .get("b")
            .clone()
            .try_into()
            .expect("Failed to convert 'b' to f64 after exit");
        assert_eq!(2.0, v);
    }
}
