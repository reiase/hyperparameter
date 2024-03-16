use std::collections::HashSet;
use std::fmt::Debug;

use crate::storage::{
    frozen_global_storage, Entry, GetOrElse, MultipleVersion, Tree, THREAD_STORAGE,
};
use crate::value::{Value, EMPTY};
use crate::xxh::XXHashable;

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

impl<T: Into<String> + Clone> From<&Vec<T>> for ParamScope {
    fn from(value: &Vec<T>) -> Self {
        let mut ps = ParamScope::default();
        value.iter().for_each(|x| ps.add(x.clone()));
        ps
    }
}

impl ParamScope {
    /// Get a parameter with a given hash key.
    pub fn get_with_hash(&self, key: u64) -> Value {
        if let ParamScope::Just(changes) = self {
            if let Some(e) = changes.get(&key) {
                match e.value() {
                    Value::Empty => {}
                    v => return v.clone(),
                }
            }
        }
        THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            ts.get_entry(key).map(|e| e.clone_value()).unwrap_or(EMPTY)
        })
    }

    /// Get a parameter with a given key.
    pub fn get<K>(&self, key: K) -> Value
    where
        K: Into<String> + Clone + XXHashable,
    {
        let hkey = key.xxh();
        self.get_with_hash(hkey)
    }

    pub fn add<T: Into<String>>(&mut self, expr: T) {
        let expr: String = expr.into();
        if let Some((k, v)) = expr.split_once('=') {
            self.put(k.to_string(), v.to_string())
        }
    }

    /// Get a list of all parameter keys.
    pub fn keys(&self) -> Vec<String> {
        let mut retval: HashSet<String> = THREAD_STORAGE.with(|ts| {
            let ts = ts.borrow();
            ts.keys().iter().cloned().collect()
        });
        if let ParamScope::Just(changes) = self {
            retval.extend(changes.values().map(|e| e.key.clone()));
        }
        retval.iter().cloned().collect()
    }

    /// Enter a new parameter scope.
    pub fn enter(&mut self) {
        THREAD_STORAGE.with(|ts| {
            let mut ts = ts.borrow_mut();
            ts.enter();
            if let ParamScope::Just(changes) = self {
                for v in changes.values() {
                    ts.put(v.key.clone(), v.value().clone());
                }
            }
        });
        *self = ParamScope::Nothing;
    }

    /// Exit the current parameter scope.
    pub fn exit(&mut self) {
        THREAD_STORAGE.with(|ts| {
            let tree = ts.borrow_mut().exit();
            *self = ParamScope::Just(tree);
        })
    }
}

/// Parameter scope operations.
pub trait ParamScopeOps<K, V> {
    fn get_or_else(&self, key: K, default: V) -> V;
    fn put(&mut self, key: K, val: V);
}

impl<V> ParamScopeOps<u64, V> for ParamScope
where
    V: Into<Value> + TryFrom<Value> + for<'a> TryFrom<&'a Value>,
{
    fn get_or_else(&self, key: u64, default: V) -> V {
        if let ParamScope::Just(changes) = self {
            if let Some(val) = changes.get(&key) {
                let r = val.value().clone().try_into();
                if r.is_ok() {
                    return r.ok().unwrap();
                }
            }
        }
        THREAD_STORAGE.with(|ts| ts.borrow_mut().get_or_else(key, default))
    }

    /// Put a parameter.
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
    K: Into<String> + Clone + XXHashable + Debug,
    V: Into<Value> + TryFrom<Value> + for<'a> TryFrom<&'a Value> + Clone,
{
    /// Get a parameter or the default value if it doesn't exist.
    fn get_or_else(&self, key: K, default: V) -> V {
        let hkey = key.xxh();
        self.get_or_else(hkey, default)
    }

    /// Put a parameter.
    fn put(&mut self, key: K, val: V) {
        let hkey = key.xxh();
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

pub fn frozen() {
    frozen_global_storage();
}

#[macro_export]
macro_rules! get_param {
    ($name:expr, $default:expr) => {{
        const CONST_KEY: &str = const_str::replace!(stringify!($name), ";", "");
        const CONST_HASH: u64 = xxhash_rust::const_xxh64::xxh64(CONST_KEY.as_bytes(), 42);
        THREAD_STORAGE.with(|ts| ts.borrow_mut().get_or_else(CONST_HASH, $default))
        // ParamScope::default().get_or_else(CONST_HASH, $default)
    }};

    ($name:expr, $default:expr, $help: expr) => {{
        const CONST_KEY: &str = const_str::replace!(stringify!($name), ";", "");
        const CONST_HASH: u64 = xxhash_rust::const_xxh64::xxh64(CONST_KEY.as_bytes(), 42);
        // ParamScope::default().get_or_else(CONST_HASH, $default)
        {
            const CONST_HELP: &str = $help;
            #[::linkme::distributed_slice(PARAMS)]
            static help: (&str, &str) = (
                CONST_KEY, CONST_HELP
            );
        }
        THREAD_STORAGE.with(|ts| ts.borrow_mut().get_or_else(CONST_HASH, $default))
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
        {
            const CONST_KEY: &str = const_str::replace!(stringify!($($key).+), ";", "");
            ps.put(CONST_KEY, $val);
        }
        with_params!(params ps; $($body)*)
    };

    (
        params $ps:expr;
        set $($key:ident).+ = $val:expr;

        $($body:tt)*
    ) => {
        {
            const CONST_KEY: &str = const_str::replace!(stringify!($($key).+), ";", "");
            $ps.put(CONST_KEY, $val);
        }
        with_params!(params $ps; $($body)*)
    };

    (
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {
        let $name = get_param!($($key).+, $default);
        with_params_readonly!($($body)*)
    };

    (
        params $ps:expr;
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {

        $ps.enter();
        let ret = {
            let $name = get_param!($($key).+, $default);

            with_params_readonly!($($body)*)
        };
        $ps.exit();
        ret

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

#[macro_export]
macro_rules! with_params_readonly {
    (
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {
        let $name = get_param!($($key).+, $default);
        with_params_readonly!($($body)*)
    };

    (
        set $($key:ident).+ = $val:expr;

        $($body:tt)*
    ) =>{
        let mut ps = ParamScope::default();
        {
            const CONST_KEY: &str = const_str::replace!(stringify!($($key).+), ";", "");
            ps.put(CONST_KEY, $val);
        }
        with_params!(params ps; $($body)*)
    };

    ($($body:tt)*) => {{
            let ret = {$($body)*};
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

    #[test]
    fn test_param_scope_with_param_readonly() {
        with_params! {
            get a_b_c = a.b.c or 1;

            assert_eq!(1, a_b_c);
        }
    }

    #[test]
    fn test_param_scope_with_param_mixed_get_set() {
        with_params! {
            get a_b_c = a.b.c or 1;
            set a.b.c = 3;
            get a_b_c = a.b.c or 2;

            assert_eq!(3, a_b_c);
        }
    }
}
