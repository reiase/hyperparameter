use std::collections::HashSet;
use std::fmt::Debug;

use crate::storage::{
    frozen_global_storage, hashstr, Entry, GetOrElse, Hashable, MultipleVersion, Tree,
    THREAD_STORAGE,
};
use crate::value::{Value, EMPTY};

#[derive(Debug)]
pub enum ParamScope {
    Nothing,
    Just(Tree),
}

impl Default for ParamScope {
    fn default() -> Self {
        ParamScope::Just(Tree::new())
    }
}

impl ParamScope {
    pub fn get_with_hash(&self, key: u64) -> Value {
        if let ParamScope::Just(changes) = self {
            if let Some(e) = changes.get(&key) {
                match e.value() {
                    Value::Empty => {}
                    v => return v.clone(),
                };
            }
        };
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            match ts.get_entry(key) {
                Some(e) => e.clone_value(),
                None => EMPTY,
            }
        })
    }

    pub fn get<K>(&self, key: K) -> Value
    where
        K: Into<String> + Clone + Hashable,
    {
        let hkey = hashstr(key);
        self.get_with_hash(hkey)
    }

    pub fn keys(&self) -> Vec<String> {
        let mut retval: HashSet<String> = THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            ts.keys().iter().cloned().collect()
        });
        if let ParamScope::Just(changes) = self {
            retval.extend(changes.values().map(|e| e.key.clone()));
        };
        retval.iter().cloned().collect()
    }

    pub fn enter(&mut self) {
        THREAD_STORAGE.with(|ts| {
            ts.borrow_mut().enter();
            if let ParamScope::Just(changes) = self {
                let mut ts = ts.borrow_mut();
                for v in changes.values() {
                    ts.put(v.key.clone(), v.value().clone());
                }
            }
        });
        *self = ParamScope::Nothing;
    }

    pub fn exit(&mut self) {
        THREAD_STORAGE.with(|ts| {
            let tree = ts.borrow_mut().exit();
            *self = ParamScope::Just(tree);
        })
    }
}

pub trait ParamScopeOps<K, V> {
    fn get_or_else(&self, key: K, default: V) -> V;
    fn put(&mut self, key: K, val: V);
}

impl<V> ParamScopeOps<u64, V> for ParamScope
where
    V: Into<Value> + TryFrom<Value>,
{
    fn get_or_else(&self, key: u64, default: V) -> V {
        if let ParamScope::Just(changes) = self {
            if let Some(val) = changes.get(&key) {
                let r = val.value().clone().try_into();
                if r.is_ok() {
                    return r.ok().unwrap();
                };
            };
        };

        THREAD_STORAGE.with(|ts| ts.borrow_mut().get_or_else(key, default))
    }

    fn put(&mut self, key: u64, val: V) {
        println!(
            "hyperparameter warning: put parameter with hashed key {}",
            key
        );
        if let ParamScope::Just(changes) = self {
            if changes.contains_key(&key) {
                changes.update(key, val);
            } else {
                changes.insert(key, Entry::new("", val));
            }
        }
    }
}

impl<K, V> ParamScopeOps<K, V> for ParamScope
where
    K: Into<String> + Clone + Hashable + Debug,
    V: Into<Value> + TryFrom<Value> + Clone,
{
    fn get_or_else(&self, key: K, default: V) -> V {
        let hkey = hashstr(key);
        self.get_or_else(hkey, default)
    }

    fn put(&mut self, key: K, val: V) {
        let hkey = hashstr(key.clone());
        if let ParamScope::Just(changes) = self {
            if changes.contains_key(&hkey) {
                changes.update(hkey, val);
            } else {
                let key: String = key.into();
                changes.insert(hkey, Entry::new(key, val));
            }
        } else {
            THREAD_STORAGE.with(|ts| ts.borrow_mut().put(key, val))
        }
    }
}

pub fn frozen_global_params() {
    frozen_global_storage();
}

#[macro_export]
macro_rules! get_param {
    ($name:expr, $default:expr) => {{
        ParamScope::default().get_or_else(stringify!($name).replace(";", ""), $default)
    }};
}

/// Define or use `hyperparameters` in a code block.
///
/// Hyperparameters are named parameters whose values control the learning process of
/// an ML model or the behaviors of an underlying machine learning system.
///
/// Hyperparameter is designed as user-friendly as global variables but overcomes two major
/// drawbacks of global variables: non-thread safety and global scope.
///
/// # A quick example
/// ```
/// use hyperparameter::*;
///
/// with_params! {   // with_params begins a new parameter scope
///     set a.b = 1; // set the value of named parameter `a.b`
///     set a.b.c = 2.0; // `a.b.c` is another parameter.
///
///     assert_eq!(1, get_param!(a.b, 0));
///
///     with_params! {   // start a new parameter scope that inherits parameters from the previous scope
///         set a.b = 2; // override parameter `a.b`
///
///         let a_b = get_param!(a.b, 0); // read parameter `a.b`, return the default value (0) if not defined
///         assert_eq!(2, a_b);
///     }
/// }
/// ```
#[macro_export]
macro_rules! with_params {
    (
        set $($key:ident).+ = $val:expr;

        $($body:tt)*
    ) =>{
        let mut ps = ParamScope::default();
        ps.put(stringify!($($key).+).replace(";", ""), $val);

        with_params!(params ps; $($body)*)
    };

    (
        params $ps:expr;
        set $($key:ident).+ = $val:expr;

        $($body:tt)*
    ) => {
        $ps.put(stringify!($($key).+).replace(";", ""), $val);

        with_params!(params $ps; $($body)*)
    };

    (
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {
        let mut ps = ParamScope::default();
        let $name = get_param!($($key).+, $default);
        with_params!(params ps; $($body)*)
    };

    (
        params $ps:expr;
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {
        let $name = get_param!($($key).+, $default);

        with_params!(params $ps; $($body)*)
    };

    (
        params $ps:expr;

        $($body:tt)*
    ) => {{
            $ps.enter();
            let ret = {$($body)*};
            $ps.exit();
            ret
    }};
}

#[cfg(test)]
mod tests {
    use crate::get_param;
    use crate::storage::{GetOrElse, THREAD_STORAGE};
    use crate::with_params;

    use super::{ParamScope, ParamScopeOps};

    #[test]
    fn test_param_scope_create() {
        let _ = ParamScope::default();
    }

    #[test]
    fn test_param_scope_put_get() {
        let mut ps = ParamScope::default();
        ps.put("1", 1);
        ps.put("2.0", 2.0);

        // check thread storage is not affected
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            assert_eq!(0, ts.get_or_else("1", 0));
            assert_eq!(0.0, ts.get_or_else("2.0", 0.0));
        });

        // check changes in param_scope
        assert_eq!(1, ps.get_or_else("1", 0));
        assert_eq!(2.0, ps.get_or_else("2.0", 0.0));
    }

    #[test]
    fn test_param_scope_enter() {
        let mut ps = ParamScope::default();
        ps.put("1", 1);
        ps.put("2.0", 2.0);

        // check thread storage is not affected
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            assert_eq!(0, ts.get_or_else("1", 0));
            assert_eq!(0.0, ts.get_or_else("2.0", 0.0));
        });

        // check changes in param_scope
        assert_eq!(1, ps.get_or_else("1", 0));
        assert_eq!(2.0, ps.get_or_else("2.0", 0.0));

        ps.enter();

        // check thread storage is affected after enter
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            assert_eq!(1, ts.get_or_else("1", 0));
            assert_eq!(2.0, ts.get_or_else("2.0", 0.0));
        });

        // check changes in param_scope
        assert_eq!(1, ps.get_or_else("1", 0));
        assert_eq!(2.0, ps.get_or_else("2.0", 0.0));

        ps.exit();
        // check thread storage is not affected after exit
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            assert_eq!(0, ts.get_or_else("1", 0));
            assert_eq!(0.0, ts.get_or_else("2.0", 0.0));
        });
        assert_eq!(1, ps.get_or_else("1", 0));
        assert_eq!(2.0, ps.get_or_else("2.0", 0.0));
    }

    #[test]
    fn test_param_scope_get_param() {
        let mut ps = ParamScope::default();
        ps.put("a.b.c", 1);

        // check thread storage is not affected
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            assert_eq!(0, ts.get_or_else("a.b.c", 0));
        });

        // check changes in param_scope
        assert_eq!(1, ps.get_or_else("a.b.c", 0));

        ps.enter();

        let x = get_param!(a.b.c, 0);
        println!("x={}", x);
    }

    #[test]
    fn test_param_scope_with_param_set() {
        with_params! {
            set a.b.c=1;
            set a.b =2;

            assert_eq!(1, get_param!(a.b.c, 0));
            assert_eq!(2, get_param!(a.b, 0));

            with_params! {
                set a.b.c=2.0;

                assert_eq!(2.0, get_param!(a.b.c, 0.0));
                assert_eq!(2, get_param!(a.b, 0));
            };

            assert_eq!(1, get_param!(a.b.c, 0));
            assert_eq!(2, get_param!(a.b, 0));
        }

        assert_eq!(0, get_param!(a.b.c, 0));
        assert_eq!(0, get_param!(a.b, 0));
    }

    #[test]
    fn test_param_scope_with_param_get() {
        with_params! {
            set a.b.c=1;

            with_params! {
                get a_b_c = a.b.c or 0;

                assert_eq!(1, a_b_c);
            };
        }
    }

    #[test]
    fn test_param_scope_with_param_set_get() {
        with_params! {
            set a.b.c = 1;
            set a.b = 2;

            with_params! {
                get a_b_c = a.b.c or 0;
                get a_b = a.b or 0;

                assert_eq!(1, a_b_c);
                assert_eq!(2, a_b);
            };
        }
    }
}
