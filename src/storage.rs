use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::entry::{Entry, EntryValue, Value};
use crate::xxh::xxhstr;

type Tree = BTreeMap<u64, Entry>;

fn hashstr<T: Into<String>>(s: T) -> u64 {
    let s: String = s.into();
    xxhstr(&s)
}

fn tree_update<T: Into<Value>>(mut tree: RefMut<Tree>, key: u64, val: T) {
    tree.entry(key).and_modify(|e| e.update(val));
}

fn tree_revision<T: Into<Value>>(mut tree: RefMut<Tree>, key: u64, val: T) {
    tree.entry(key).and_modify(|e| e.revision(val));
}

fn tree_rollback(mut tree: RefMut<Tree>, key: u64) {
    if let Some(need_del) = tree.get_mut(&key).map(|e| e.rollback().is_err()) {
        if need_del {
            tree.remove(&key);
        }
    }
}

pub struct StorageManager {
    pub tls: Rc<RefCell<Tree>>,
    pub stack: Vec<RefCell<HashSet<u64>>>,
}

impl StorageManager {
    pub fn put_key<T: Into<String>>(&mut self, key: T) {
        let key = hashstr(key);
        if let Some(hash) = self.stack.last() {
            hash.borrow_mut().insert(key);
        }
    }
    pub fn put_hkey(&mut self, key: u64) {
        if let Some(hash) = self.stack.last() {
            hash.borrow_mut().insert(key);
        }
    }
}

thread_local! {
    pub static MGR: RefCell<StorageManager> = init_storage_manager();
}

pub fn init_storage_manager() -> RefCell<StorageManager> {
    let mut tree = Tree::new();
    global_storage_get(&mut tree);
    let sm = RefCell::new(StorageManager {
        tls: Rc::new(RefCell::new(tree)),
        stack: Vec::new(),
    });
    sm.borrow_mut().stack.push(RefCell::new(HashSet::new()));

    return sm;
}

lazy_static! {
    static ref GLOBAL_STORAGE: Mutex<u64> = {
        let tree = Box::new(Tree::new());
        Mutex::new(Box::into_raw(tree) as u64)
    };
}

fn global_storage_set(t: &Tree) {
    GLOBAL_STORAGE
        .lock()
        .and_then(|v| unsafe {
            let ptr = v.clone() as *mut Tree;
            match ptr.as_mut() {
                Some(tree) => {
                    tree.clear();
                    tree.clone_from(t);
                }
                None => todo!(),
            };
            Ok(())
        })
        .unwrap();
}

pub fn frozen_as_global_storage() {
    MGR.with(|mgr| {
        let t = mgr.borrow().tls.borrow().clone();
        global_storage_set(&t);
    });
}

fn global_storage_get(t: &mut Tree) {
    GLOBAL_STORAGE
        .lock()
        .and_then(|v| unsafe {
            let ptr = v.clone() as *mut Tree;
            match ptr.as_mut() {
                Some(tree) => {
                    t.clear();
                    t.clone_from(tree);
                }
                None => todo!(),
            };
            Ok(())
        })
        .unwrap();
}

#[derive(Debug)]
pub struct Storage {
    pub parent: Rc<RefCell<Tree>>,
    pub tree: RefCell<Tree>,
    pub isview: i32,
}
unsafe impl Send for Storage {}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            parent: MGR.with(|mgr| mgr.borrow().tls.clone()),
            tree: RefCell::new(Tree::new()),
            isview: 0,
        }
    }

    pub fn new_empty() -> Storage {
        Storage {
            parent: Rc::new(RefCell::new(Tree::new())),
            tree: RefCell::new(Tree::new()),
            isview: 0,
        }
    }

    fn tree(&self) -> Ref<Tree> {
        self.tree.borrow()
    }

    pub fn enter(&mut self) {
        MGR.with(|m| {
            for (k, v) in self.tree().iter() {
                if m.borrow_mut().tls.borrow().contains_key(&k) {
                    tree_revision(m.borrow_mut().tls.borrow_mut(), *k, v.clone_value());
                } else {
                    m.borrow_mut().tls.borrow_mut().insert(*k, v.clone());
                }
            }
            let keys = self.tree().keys().cloned().collect();
            m.borrow_mut().stack.push(RefCell::new(keys));
        });
        self.isview += 1;
    }

    pub fn exit(&mut self) {
        MGR.with(|m| {
            let mut m = m.borrow_mut();
            if let Some(keys) = m.stack.pop() {
                keys.borrow()
                    .iter()
                    .for_each(|k| tree_rollback(m.tls.borrow_mut(), *k));
            }
        });
        self.isview -= 1;
    }

    pub fn get_by_hash(&self, key: u64) -> Option<Value> {
        if self.isview == 0 {
            if let Some(e) = self.tree().get(&key) {
                return Some(e.clone_value());
            } else if let Some(e) = self.parent.borrow().get(&key) {
                return Some(e.clone_value());
            }
        } else {
            if let Some(e) = self.parent.borrow().get(&key) {
                return Some(e.clone_value());
            }
        }
        return None;
    }

    pub fn put_by_hash(&mut self, key: u64, val: Entry) {
        self.tree.borrow_mut().insert(key, val);
    }

    pub fn del_by_hash(&mut self, key: u64) {
        self.tree.borrow_mut().remove(&key);
    }

    pub fn get<T: Into<String>>(&self, key: T) -> Option<Value> {
        let hkey = hashstr(key);
        self.get_by_hash(hkey)
    }

    pub fn put<T: Into<String>, V: Into<Value> + Clone>(&mut self, key: T, val: V) {
        let key: String = key.into();
        let hkey = hashstr(&key);
        self.put_by_hash(
            hkey,
            Entry {
                key: key.clone(),
                val: EntryValue::Single(val.clone().into()),
            },
        );
        if self.isview > 0 {
            if self.parent.borrow().contains_key(&hkey) {
                tree_update(self.parent.borrow_mut(), hkey, val.clone())
            } else {
                self.parent.borrow_mut().insert(
                    hkey,
                    Entry {
                        key: key.clone(),
                        val: EntryValue::Single(val.clone().into()),
                    },
                );
                MGR.with(|mgr: &RefCell<StorageManager>| mgr.borrow_mut().put_hkey(hkey));
            }
        }
    }

    pub fn del<T: Into<String>>(&mut self, key: T) {
        let hkey = hashstr(key);
        self.del_by_hash(hkey);
    }

    pub fn rollback<T: Into<String>>(&mut self, key: T) {
        let hkey = hashstr(key);
        tree_rollback(self.tree.borrow_mut(), hkey);
    }

    pub fn keys(&self) -> Vec<String> {
        let mut allkey = HashSet::<String>::new();
        for v in self.parent.borrow().values() {
            allkey.insert(v.key.clone());
        }
        for v in self.tree.borrow().values() {
            allkey.insert(v.key.clone());
        }
        allkey.iter().cloned().collect()
    }
}

impl Storage {
    pub fn get_or_else<T: Into<Value> + TryFrom<Value>>(&self, key: u64, dval: T) -> T {
        if let Some(val) = self.get_by_hash(key) {
            match val.try_into() {
                Ok(v) => v,
                Err(_) => dval,
            }
        } else {
            dval
        }
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
