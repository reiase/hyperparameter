use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::value::EMPTY;
use crate::value::Value;
use crate::value::VersionedValue;
use crate::xxh::xxhstr;

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

pub type Tree = BTreeMap<u64, Entry>;

pub trait MultipleVersion<K> {
    fn update<V: Into<Value>>(&mut self, key: K, val: V);
    fn revision<V: Into<Value>>(&mut self, key: K, val: V);
    fn rollback(&mut self, key: K);
}

impl MultipleVersion<u64> for Tree {
    fn update<V: Into<Value>>(&mut self, key: u64, val: V) {
        self.entry(key).and_modify(|e| {
            e.val.update(val);
        });
    }

    fn revision<V: Into<Value>>(&mut self, key: u64, val: V) {
        self.entry(key).and_modify(|e| e.val.revision(val));
    }

    fn rollback(&mut self, key: u64) {
        if let Some(e) = self.get_mut(&key) {
            if !e.val.rollback() {
                self.remove(&key);
            }
        }
    }
}

pub fn hashstr<T: Into<String>>(s: T) -> u64 {
    let s: String = s.into();
    xxhstr(&s)
}

thread_local! {
    pub static THREAD_STORAGE: RefCell<Storage> = create_thread_storage();
}

pub fn create_thread_storage() -> RefCell<Storage> {
    let ts = RefCell::new(Storage::default());
    ts.borrow_mut()
        .tree
        .clone_from(&GLOBAL_STORAGE.lock().unwrap().tree);
    ts
}

lazy_static! {
    static ref GLOBAL_STORAGE: Mutex<Storage> = Mutex::new(Storage::default());
}

pub fn frozen_global_storage() {
    THREAD_STORAGE.with(|ts| {
        GLOBAL_STORAGE
            .lock()
            .unwrap()
            .tree
            .clone_from(&ts.borrow().tree);
    });
}

#[derive(Debug)]
pub struct Storage {
    pub tree: Tree,
    pub history: Vec<HashSet<u64>>,
}

unsafe impl Send for Storage {}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            tree: Tree::new(),
            history: vec![HashSet::new()],
        }
    }
}

impl Storage {
    pub fn enter(&mut self) {
        self.history.push(HashSet::new());
    }

    pub fn exit(&mut self) -> Tree {
        let mut changes = Tree::new();
        for key in self.history.pop().unwrap() {
            changes.insert(key, self.tree.get(&key).unwrap().shallow());
            self.tree.rollback(key);
        }
        changes
    }

    pub fn get_entry(&self, key: u64) -> Option<&Entry> {
        self.tree.get(&key)
    }

    pub fn put_entry(&mut self, key: u64, entry: Entry) -> Option<Entry> {
        self.tree.insert(key, entry)
    }

    pub fn del_entry(&mut self, key: u64) {
        self.tree.remove(&key);
    }

    pub fn get<T: Into<String>>(&self, key: T) -> &Value {
        let hkey = hashstr(key);
        if let Some(e) = self.tree.get(&hkey) {
            return e.value();
        }
        return &EMPTY;
    }

    pub fn put<T: Into<String>, V: Into<Value> + Clone>(&mut self, key: T, val: V) {
        let key: String = key.into();
        let hkey = hashstr(&key);
        if self.history.last().unwrap().contains(&hkey) {
            self.tree.update(hkey, val);
        } else {
            if self.tree.contains_key(&hkey) {
                self.tree.revision(hkey, val);
            } else {
                self.tree.insert(
                    hkey,
                    Entry {
                        key: key,
                        val: VersionedValue::Single(val.into()),
                    },
                );
            }
            self.history.last_mut().unwrap().insert(hkey);
        }
    }

    pub fn del<T: Into<String>>(&mut self, key: T) {
        let hkey = hashstr(key);
        if self.history.last().unwrap().contains(&hkey) {
            self.tree.update(hkey, None::<i32>);
        } else {
            self.tree.revision(hkey, None::<i32>);
            self.history.last_mut().unwrap().insert(hkey);
        }
    }

    // pub fn rollback<T: Into<String>>(&mut self, key: T) {
    //     let hkey = hashstr(key);
    //     self.tree.rollback(hkey);
    // }

    pub fn keys(&self) -> Vec<String> {
        self.tree
            .values()
            .filter(|x| match x.value() {
                Value::Empty => false,
                _ => true,
            })
            .map(|x| x.key.clone())
            .collect()
    }
}

pub trait Hashable {}

impl Hashable for String {}

impl Hashable for &String {}

impl Hashable for &str {}

impl Hashable for str {}

pub trait GetOrElse<K, T> {
    fn get_or_else(&self, key: K, dval: T) -> T;
}

impl<T> GetOrElse<u64, T> for Storage
    where
        T: Into<Value> + TryFrom<Value>,
{
    fn get_or_else(&self, key: u64, dval: T) -> T {
        if let Some(val) = self.tree.get(&key) {
            match val.value().clone().try_into() {
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
        K: Into<String> + Hashable,
        T: Into<Value> + TryFrom<Value>,
{
    fn get_or_else(&self, key: K, dval: T) -> T {
        let hkey = hashstr(key);
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

        let v: i64 = s.get("1").clone().try_into().unwrap();
        assert_eq!(1, v);

        let v: f64 = s.get("2.0").clone().try_into().unwrap();
        assert_eq!(2.0, v);

        let v: String = s.get("str").clone().try_into().unwrap();
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
        let v: i64 = s0.get("a").clone().try_into().unwrap();
        assert_eq!(1, v);

        let v: f64 = s0.get("b").clone().try_into().unwrap();
        assert_eq!(2.0, v);

        s0.put("a", 2);
        s0.put("b", 3.0);
        let v: i64 = s0.get("a").clone().try_into().unwrap();
        assert_eq!(2, v);

        let v: f64 = s0.get("b").clone().try_into().unwrap();
        assert_eq!(3.0, v);

        s0.exit();
        // check parameter "a" and "b"
        let v: i64 = s0.get("a").clone().try_into().unwrap();
        assert_eq!(1, v);

        let v: f64 = s0.get("b").clone().try_into().unwrap();
        assert_eq!(2.0, v);
    }
}
