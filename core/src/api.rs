use std::collections::HashSet;
use std::fmt::Debug;

use crate::storage::{
    frozen_global_storage, Entry, GetOrElse, MultipleVersion, Params, THREAD_STORAGE,
};
use crate::value::{Value, EMPTY};
use crate::xxh::XXHashable;

/// ParameterScope
///
/// `ParameterScope` is a data structure that stores the current set of named parameters
/// and their values. `ParameterScope` is used to manage the scope of named parameters,
/// allowing parameters to be defined and used within a specific scope,
/// and then restored to the previous scope when the scope is exited.
///
/// The parameter scope can be used to implement a variety of features, such
/// as named parameters, default parameter values, and parameter inheritance.
#[derive(Debug, Clone)]
pub enum ParamScope {
    /// No parameters are defined in the current scope.
    Nothing,
    /// The current scope contains a set of named parameters stored in `Params`.
    Just(Params),
}

impl Default for ParamScope {
    fn default() -> Self {
        ParamScope::Just(Params::new())
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

    /// Enter a parameter scope and return a guard that will exit on drop (panic-safe).
    pub fn enter_guard(&mut self) -> ParamScopeGuard<'_> {
        self.enter();
        ParamScopeGuard {
            scope: self,
            active: true,
        }
    }
}

/// RAII guard that restores the previous parameter scope even if a panic occurs.
pub struct ParamScopeGuard<'a> {
    scope: &'a mut ParamScope,
    active: bool,
}

impl<'a> Drop for ParamScopeGuard<'a> {
    fn drop(&mut self) {
        if self.active {
            self.scope.exit();
            self.active = false;
        }
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
                if let Ok(v) = r {
                    return v;
                }
            }
        }
        THREAD_STORAGE.with(|ts| ts.borrow_mut().get_or_else(key, default))
    }

    /// Put a parameter.
    fn put(&mut self, key: u64, val: V) {
        if let ParamScope::Just(changes) = self {
            if let std::collections::btree_map::Entry::Vacant(e) = changes.entry(key) {
                e.insert(Entry::new("", val));
            } else {
                changes.update(key, val);
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
            // if changes.contains_key(&hkey) {
            //     changes.update(hkey, val);
            // } else {
            //     let key: String = key.into();
            //     changes.insert(hkey, Entry::new(key, val));
            // }
            if let std::collections::btree_map::Entry::Vacant(e) = changes.entry(hkey) {
                let key: String = key.into();
                e.insert(Entry::new(key, val));
            } else {
                changes.update(hkey, val);
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
            static help: (&str, &str) = (CONST_KEY, CONST_HELP);
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
    ) => {{
        let mut ps = ParamScope::default();
        {
            const CONST_KEY: &str = const_str::replace!(stringify!($($key).+), ";", "");
            ps.put(CONST_KEY, $val);
        }
        with_params!(params ps; $($body)*)
    }};

    (
        params $ps:expr;
        set $($key:ident).+ = $val:expr;

        $($body:tt)*
    ) => {{
        {
            const CONST_KEY: &str = const_str::replace!(stringify!($($key).+), ";", "");
            $ps.put(CONST_KEY, $val);
        }
        with_params!(params $ps; $($body)*)
    }};

    (
        params $ps:expr;
        params $nested:expr;

        $($body:tt)*
    ) => {{
        let mut __hp_ps = $ps;
        let _hp_guard = __hp_ps.enter_guard();
        let mut __hp_nested = $nested;
        let ret = with_params!(params __hp_nested; $($body)*);
        ret
    }};

    (
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {{
        let $name = get_param!($($key).+, $default);
        with_params_readonly!($($body)*)
    }};

    (
        $(#[doc = $doc:expr])*
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {{
        let $name = get_param!($($key).+, $default, $($doc)*);
        with_params_readonly!($($body)*)
    }};

    (
        params $ps:expr;
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {{
        let mut __hp_ps = $ps;
        let _hp_guard = __hp_ps.enter_guard();
        let ret = {{
            let $name = get_param!($($key).+, $default);

            with_params_readonly!($($body)*)
        }};
        ret
    }};

    (
        params $ps:expr;

        $($body:tt)*
    ) => {{
            let mut __hp_ps = $ps;
            let _hp_guard = __hp_ps.enter_guard();
            let ret = {$($body)*};
            ret
    }};

    ($($body:tt)*) => {{
        let ret = {$($body)*};
        ret
    }};
}

#[macro_export]
macro_rules! with_params_readonly {
    (
        get $name:ident = $($key:ident).+ or $default:expr;

        $($body:tt)*
    ) => {{
        let $name = get_param!($($key).+, $default);
        with_params_readonly!($($body)*)
    }};

    (
        set $($key:ident).+ = $val:expr;

        $($body:tt)*
    ) => {{
        let mut ps = ParamScope::default();
        {
            const CONST_KEY: &str = const_str::replace!(stringify!($($key).+), ";", "");
            ps.put(CONST_KEY, $val);
        }
        with_params!(params ps; $($body)*)
    }};

    ($($body:tt)*) => {{
            let ret = {$($body)*};
            ret
    }};
}

#[cfg(test)]
mod tests {
    use crate::storage::{GetOrElse, THREAD_STORAGE};

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
            get _a_b_c = a.b.c or 1;
            set a.b.c = 3;
            get a_b_c = a.b.c or 2;

            assert_eq!(3, a_b_c);
        }
    }
}

// FILEPATH: /home/reiase/workspace/hyperparameter/core/src/api.rs
// BEGIN: test_code

#[cfg(test)]
mod test_param_scope {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_param_scope_default() {
        let ps = ParamScope::default();
        match ps {
            ParamScope::Just(_) => assert!(true),
            _ => assert!(false, "Default ParamScope should be ParamScope::Just"),
        }
    }

    #[test]
    fn test_param_scope_from_vec() {
        let vec = vec!["param1=value1", "param2=value2"];
        let ps: ParamScope = (&vec).into();
        match ps {
            ParamScope::Just(params) => {
                assert_eq!(
                    params
                        .get(&"param1".xxh())
                        .expect("param1 should exist")
                        .value(),
                    &Value::from("value1")
                );
                assert_eq!(
                    params
                        .get(&"param2".xxh())
                        .expect("param2 should exist")
                        .value(),
                    &Value::from("value2")
                );
            }
            _ => panic!("ParamScope should be ParamScope::Just"),
        }
    }

    #[test]
    fn test_param_scope_get_with_hash() {
        let mut ps = ParamScope::default();
        ps.add("param=value");
        let value = ps.get_with_hash("param".xxh());
        assert_eq!(value, Value::from("value"));
    }

    #[test]
    fn test_param_scope_get() {
        let mut ps = ParamScope::default();
        ps.add("param=value");
        let value: String = ps
            .get("param")
            .try_into()
            .expect("Failed to convert param to String");
        assert_eq!(value, "value");
    }

    #[test]
    fn test_param_scope_add() {
        let mut ps = ParamScope::default();
        ps.add("param=value");
        match ps {
            ParamScope::Just(params) => {
                assert_eq!(
                    params
                        .get(&"param".xxh())
                        .expect("param should exist")
                        .value(),
                    &Value::from("value")
                );
            }
            _ => panic!("ParamScope should be ParamScope::Just"),
        }
    }

    #[test]
    fn test_param_scope_keys() {
        let mut ps = ParamScope::default();
        ps.add("param=value");
        let keys = ps.keys();
        assert_eq!(keys, vec!["param"]);
    }

    #[test]
    fn test_param_scope_enter_exit() {
        let mut ps = ParamScope::default();
        ps.add("param=value");
        ps.enter();
        match ps {
            ParamScope::Nothing => assert!(true),
            _ => assert!(
                false,
                "ParamScope should be ParamScope::Nothing after enter"
            ),
        }
        ps.exit();
        match ps {
            ParamScope::Just(_) => assert!(true),
            _ => assert!(false, "ParamScope should be ParamScope::Just after exit"),
        }
    }
}

// END: test_code
