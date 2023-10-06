use crate::entry::{Entry, Value, EMPTY};
use crate::storage::{hashstr, GetOrElse, Hashable, MultipleVersion, Tree, THREAD_STORAGE, frozen_global_storage};

pub enum ParamScope {
    Nothing,
    Just(Tree),
}

impl ParamScope {
    pub fn new() -> ParamScope {
        ParamScope::Just(Tree::new())
    }

    pub fn get_with_hash<'a, 'b>(&self, key: u64) -> Value {
        if let ParamScope::Just(changes) = self {
            if let Some(e) = changes.get(&key) {
                match e.get() {
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

    pub fn enter(&mut self) {
        THREAD_STORAGE.with(|ts| {
            ts.borrow_mut().enter();
            if let ParamScope::Just(changes) = self {
                let mut ts = ts.borrow_mut();
                for (_, v) in changes {
                    ts.put(v.key.clone(), v.get().clone());
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
                let r = val.get().clone().try_into();
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
    K: Into<String> + Clone + Hashable,
    V: Into<Value> + TryFrom<Value>,
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
        }
    }
}

pub fn frozen_global_params() {
    frozen_global_storage();
}

#[macro_export]
macro_rules! get_param {
    ($name:expr, $default:expr) => {{
        ParamScope::new().get_or_else(stringify!($name).replace(";", ""), $default)
    }};
}

#[macro_export]
macro_rules! with_params {
    (
        set $($key:ident).+ = $val:expr;
        
        $($body:tt)*
    ) =>{
        let mut ps = ParamScope::new();
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
        let mut ps = ParamScope::new();
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
    use crate::storage::{GetOrElse, THREAD_STORAGE};

    use super::{ParamScope, ParamScopeOps};
    use crate::get_param;
    use crate::with_params;

    #[test]
    fn test_param_scope_create() {
        let _ = ParamScope::new();
    }

    #[test]
    fn test_param_scope_put_get() {
        let mut ps = ParamScope::new();
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
        let mut ps = ParamScope::new();
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
        let mut ps = ParamScope::new();
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
        };
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
        };
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
        };
    }
}
