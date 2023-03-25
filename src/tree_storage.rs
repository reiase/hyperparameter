use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;

use std::hash::{Hash, Hasher};

use crate::entry::Entry;
use crate::entry::EntryValue;
use crate::entry::Value;

pub fn strhash<T: Hash>(s: &T) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub struct TreeStorage {
    pub storage: BTreeMap<u64, Entry>,
}

impl TreeStorage {
    pub fn new() -> TreeStorage {
        TreeStorage {
            storage: BTreeMap::new(),
        }
    }

    pub fn get_by_hash(&self, key: u64) -> Option<&Value> {
        match self.storage.get(&key) {
            Some(e) => Some(e.get()),
            None => None,
        }
    }

    pub fn put_by_hash(&mut self, key: u64, val: Entry) {
        self.storage.insert(key, val);
    }

    pub fn del_by_hash(&mut self, key: u64) {
        self.storage.remove(&key);
    }

    pub fn revision_by_hash<V: Into<Value>>(&mut self, key: u64, val: V) {
        self.storage.entry(key).and_modify(|e| {
            e.revision(val);
        });
    }

    pub fn rollback_by_hash(&mut self, key: u64) {
        let mut need_del = true;
        self.storage.get_mut(&key).map(|e| {
            match e.rollback() {
                Ok(_) => need_del = false,
                Err(_) => need_del = true,
            };
        });
        if need_del {
            self.storage.remove(&key);
        }
    }

    pub fn get<T: Into<String>>(&self, key: T) -> Option<&Value> {
        let key: String = key.into();
        let hkey = strhash(&key);
        self.get_by_hash(hkey)
    }

    pub fn put<T: Into<String>, V: Into<Value>>(&mut self, key: T, val: V) {
        let key: String = key.into();
        let hkey = strhash(&key);
        if self.storage.contains_key(&hkey) {
            self.revision_by_hash(hkey, val);
        } else {
            self.put_by_hash(
                hkey,
                Entry {
                    key: key,
                    val: EntryValue::Single(val.into()),
                },
            );
        }
    }

    pub fn del<T: Into<String>>(&mut self, key: T) {
        let key: String = key.into();
        let hkey = strhash(&key);
        self.del_by_hash(hkey);
    }

    pub fn rollback<T: Into<String>>(&mut self, key: T) {
        let key: String = key.into();
        let hkey = strhash(&key);
        self.rollback_by_hash(hkey);
    }
}

#[cfg(test)]
mod tests {
    extern crate rspec;
    use std::ffi::c_void;

    use crate::tree_storage::TreeStorage;

    use super::Entry;
    use super::Value;
    #[test]
    fn test() {
        rspec::run(&rspec::describe(
            "basic tree storage operations",
            (),
            |ctx| {
                ctx.specify("create tree storage", |ctx| {
                    ctx.it("use default creator", |_| {
                        let _ = TreeStorage::new();
                    });
                });

                ctx.specify("kv operation by hash", |ctx| {
                    ctx.it("put int", |_| {
                        let mut s = TreeStorage::new();
                        s.put_by_hash(1, Entry::new("1", Value::Int(1)));
                        assert_eq!(s.get_by_hash(1).unwrap(), &Value::Int(1));
                    });
                    ctx.it("put float", |_| {
                        let mut s = TreeStorage::new();
                        s.put_by_hash(1, Entry::new("1", Value::Float(1.0)));
                        assert_eq!(s.get_by_hash(1).unwrap(), &Value::Float(1.0));
                    });
                    ctx.it("put text", |_| {
                        let mut s = TreeStorage::new();
                        s.put_by_hash(1, Entry::new("1", Value::from("a")));
                        assert_eq!(s.get_by_hash(1).unwrap(), &Value::from("a"));
                    });
                    ctx.it("put bool", |_| {
                        let mut s = TreeStorage::new();
                        s.put_by_hash(1, Entry::new("1", Value::from(true)));
                        assert_eq!(s.get_by_hash(1).unwrap(), &Value::from(true));
                    });
                    ctx.it("put ptr", |_| {
                        let mut s = TreeStorage::new();
                        s.put_by_hash(1, Entry::new("1", Value::from(0 as *mut c_void)));
                        assert_eq!(s.get_by_hash(1).unwrap(), &Value::from(0 as *mut c_void));
                    });
                });

                ctx.specify("kv operations", |ctx| {
                    ctx.it("get with an empty key", |_| {
                        let s = TreeStorage::new();
                        assert_eq!(s.get("a"), None);
                    });
                    ctx.it("del an empty key", |_| {
                        let mut s = TreeStorage::new();
                        s.del("a");
                    });
                    ctx.it("put&get with keys", |_| {
                        let mut s = TreeStorage::new();
                        s.put("a", 1);
                        let value = s.get("a").unwrap();
                        assert_eq!(value, &Value::from(1));
                    });
                    ctx.it("put/del/get with keys", |_| {
                        let mut s = TreeStorage::new();

                        // put
                        s.put("a", 1);

                        // get
                        assert_eq!(s.get("a").unwrap(), &Value::from(1));

                        // del
                        s.del("a");

                        // get deleted key
                        assert_eq!(s.get("a"), None);
                    });
                });

                ctx.specify("versioned operation", |ctx| {
                    ctx.it("support revision", |_| {
                        let mut s = TreeStorage::new();
                        // put
                        s.put("a", 1);

                        // put with revision
                        s.put("a", "a");

                        // get revisioned value
                        assert_eq!(s.get("a").unwrap(), &Value::from("a"));

                        // rollback
                        s.rollback("a");

                        // get original value
                        assert_eq!(s.get("a").unwrap(), &Value::from(1));

                        // rollback
                        s.rollback("a");

                        // get empty value
                        assert_eq!(s.get("a"), None);
                    });
                });
            },
        ));
    }
}
