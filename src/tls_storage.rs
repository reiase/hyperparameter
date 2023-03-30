use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::{
    entry::{Entry, Value},
    tree_storage::{strhash, TreeStorage},
};

pub struct StorageManager {
    pub base: Rc<RefCell<TreeStorage>>,
    pub current: Vec<*mut Storage>,
}

impl StorageManager {
    pub fn new() -> StorageManager {
        StorageManager {
            base: Rc::new(RefCell::new(TreeStorage::new())),
            current: Vec::new(),
        }
    }
}

thread_local! {
    pub static EMPTY: RefCell<Storage> = RefCell::new(Storage::init());
    pub static MGR: RefCell<StorageManager> = init_tls_storage();
}

pub fn init_tls_storage() -> RefCell<StorageManager> {
    let sm = RefCell::new(StorageManager::new());
    EMPTY.with(|s| {
        let ptr: *mut Storage = &mut *s.borrow_mut();
        sm.borrow_mut().current.push(ptr);
    });
    sm
}

pub struct Storage {
    pub parent: Rc<RefCell<TreeStorage>>,
    pub tree: TreeStorage,
}
unsafe impl Send for Storage {}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            parent: MGR.with(|mgr| mgr.borrow().base.clone()),
            tree: TreeStorage::new(),
        }
    }

    pub fn init() -> Storage {
        Storage {
            parent: Rc::new(RefCell::new(TreeStorage::new())),
            tree: TreeStorage::new(),
        }
    }

    pub fn enter(&mut self) {
        // commit into storage manager
        MGR.with(|mgr| {
            let mut storage = mgr.borrow_mut();
            for (k, v) in self.tree.storage.iter() {
                if storage.base.borrow_mut().storage.contains_key(&k) {
                    storage
                        .base
                        .borrow_mut()
                        .revision_by_hash(*k, v.get().clone());
                } else {
                    let v = v.clone();
                    storage.base.borrow_mut().put_by_hash(*k, v);
                }
            }
            let ptr: *mut Storage = &mut *self;
            storage.current.push(ptr);
        });
    }

    pub fn exit(&mut self) {
        MGR.with(|mgr| {
            let mut storage = mgr.borrow_mut();
            for (k, _) in self.tree.storage.iter() {
                storage.base.borrow_mut().rollback_by_hash(*k);
            }
            storage.current.pop();
        });
    }

    pub fn get_by_hash(&self, key: u64) -> Option<Value> {
        match self.tree.get_by_hash(key) {
            Some(e) => Some(e.clone()),
            None => {
                let parent = self.parent.borrow_mut();
                match parent.get_by_hash(key) {
                    Some(e) => Some(e.clone()),
                    None => None,
                }
            }
        }
    }

    pub fn put_by_hash(&mut self, key: u64, val: Entry) {
        self.tree.put_by_hash(key, val);
    }

    pub fn del_by_hash(&mut self, key: u64) {
        self.tree.del_by_hash(key);
    }

    pub fn revision_by_hash<V: Into<Value>>(&mut self, key: u64, val: V) {
        self.tree.revision_by_hash(key, val);
    }

    pub fn rollback_by_hash(&mut self, key: u64) {
        self.tree.rollback_by_hash(key);
    }

    pub fn get<T: Into<String>>(&self, key: T) -> Option<Value> {
        let key: String = key.into();
        let hkey = strhash(&key);
        match self.tree.get_by_hash(hkey) {
            Some(v) => Some(v.clone()),
            None => {
                let parent = self.parent.borrow_mut();
                match parent.get_by_hash(hkey) {
                    Some(e) => Some(e.clone()),
                    None => None,
                }
            }
        }
    }

    pub fn put<T: Into<String>, V: Into<Value>>(&mut self, key: T, val: V) {
        self.tree.put(key, val);
    }

    pub fn del<T: Into<String>>(&mut self, key: T) {
        self.tree.del(key);
    }

    pub fn rollback<T: Into<String>>(&mut self, key: T) {
        self.tree.rollback(key);
    }

    pub fn keys(&self) -> Vec<String> {
        // let mut res = Vec::<String>::new();
        let mut allkey = HashSet::<String>::new();
        for v in self.parent.borrow_mut().storage.values() {
            allkey.insert(v.key.clone());
        }
        for v in self.tree.storage.values() {
            allkey.insert(v.key.clone());
        }
        let res: Vec<String> = allkey.iter().cloned().collect();
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::entry::Value;

    use super::Storage;

    #[test]
    fn test() {
        rspec::run(&rspec::describe("basic storage operations", (), |ctx| {
            ctx.specify("create storage", |ctx| {
                ctx.it("create from None", |_| {
                    let _ = Storage::new();
                });
            });
        }));

        rspec::run(&rspec::describe("enter/exit operations", (), |ctx| {
            ctx.specify("enter", |ctx| {
                let mut s0 = Storage::new();
                s0.put("a", 1);
                s0.put("b", 2);
                s0.enter();

                // storage after enter
                let s1 = Storage::new();
                assert_eq!(s1.get("a").unwrap(), Value::from(1));
                assert_eq!(s1.get("b").unwrap(), Value::from(2));
                assert_eq!(s1.get("c"), None);

                s0.exit();
                let s1 = Storage::new();
                assert_eq!(s1.get("a"), None);
                assert_eq!(s1.get("b"), None);
            });
        }));
    }
}
