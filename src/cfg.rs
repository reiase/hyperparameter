pub use crate::api::frozen;
pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;
pub use crate::storage::GetOrElse;
pub use crate::storage::THREAD_STORAGE;
pub use crate::value::Value;
pub use crate::xxh::XXHashable;

use config;

pub trait AsParamScope {
    fn param_scope(self: &Self) -> ParamScope;
}

impl AsParamScope for config::Config {
    fn param_scope(self: &Self) -> ParamScope {
        let mut ps = ParamScope::default();
        fn unpack(ps: &mut ParamScope, prefix: Option<String>, value: config::Value) -> () {
            match (prefix, value.kind) {
                (None, config::ValueKind::Table(v)) => v.iter().for_each(|(k, v)| {
                    unpack(ps, Some(k.to_string()), v.clone());
                }),
                (Some(k), config::ValueKind::Boolean(v)) => ps.put(k, v),
                (Some(k), config::ValueKind::I64(v)) => ps.put(k, v),
                (Some(k), config::ValueKind::Float(v)) => ps.put(k, v),
                (Some(k), config::ValueKind::String(v)) => ps.put(k, v),
                (Some(prefix), config::ValueKind::Table(v)) => v.iter().for_each(|(k, v)| {
                    unpack(ps, Some(format!("{}.{}", prefix, k.to_string())), v.clone());
                }),
                _ => todo!(),
            };
        }
        unpack(&mut ps, None, self.cache.clone());

        ps
    }
}

#[cfg(test)]
mod tests {
    use config::ConfigError;

    // use crate::with_params;
    use crate::*;

    use super::AsParamScope;

    #[test]
    fn test_create_param_scope_from_config() -> Result<(), ConfigError> {
        let mut cfg = config::Config::builder()
            .set_default("a", 1)?
            .set_default("b", "2")?
            .set_default(
                "foo",
                config::Config::builder()
                    .set_default("a", 11)?
                    .set_default("b", "22")?
                    .build()?
                    .cache
                    .clone()
                    .into_table()?,
            )?
            .build()?
            .param_scope();
        with_params! {
            params cfg;

            with_params! {
                get a = a or 0i64;
                get b = b or String::from("2");
                get foo_a = foo.a or 0i64;

                assert_eq!(1, a);
                assert_eq!("2", b);
                assert_eq!(11, foo_a);

                println!("a = {}", a);
                println!("b = {}", b);
                println!("foo.a= {}", foo_a);
            }
        }
        Ok(())
    }
}
