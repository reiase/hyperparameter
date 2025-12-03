pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;

pub trait AsParamScope {
    fn param_scope(&self) -> ParamScope;
}

impl AsParamScope for config::Config {
    fn param_scope(&self) -> ParamScope {
        let mut ps = ParamScope::default();
        fn unpack(ps: &mut ParamScope, prefix: Option<String>, value: config::Value) {
            match (prefix, value.kind) {
                // Root level table - unpack all entries
                (None, config::ValueKind::Table(v)) => v.iter().for_each(|(k, v)| {
                    unpack(ps, Some(k.to_string()), v.clone());
                }),
                // Nested table - unpack with prefix
                (Some(prefix), config::ValueKind::Table(v)) => v.iter().for_each(|(k, v)| {
                    unpack(ps, Some(format!("{}.{}", prefix, k)), v.clone());
                }),
                // Primitive types with prefix
                (Some(k), config::ValueKind::Boolean(v)) => ps.put(k, v),
                (Some(k), config::ValueKind::I64(v)) => ps.put(k, v),
                (Some(k), config::ValueKind::Float(v)) => ps.put(k, v),
                (Some(k), config::ValueKind::String(v)) => ps.put(k, v),
                // Additional integer types with prefix
                (Some(k), config::ValueKind::I128(v)) => {
                    // Convert i128 to i64 if possible, otherwise to string
                    if v >= i64::MIN as i128 && v <= i64::MAX as i128 {
                        ps.put(k, v as i64);
                    } else {
                        ps.put(k, v.to_string());
                    }
                }
                (Some(k), config::ValueKind::U64(v)) => {
                    // Convert u64 to i64 if possible, otherwise to string
                    if v <= i64::MAX as u64 {
                        ps.put(k, v as i64);
                    } else {
                        ps.put(k, v.to_string());
                    }
                }
                (Some(k), config::ValueKind::U128(v)) => {
                    // Convert u128 to i64 if possible, otherwise to string
                    if v <= i64::MAX as u128 {
                        ps.put(k, v as i64);
                    } else {
                        ps.put(k, v.to_string());
                    }
                }
                // Array type - convert to comma-separated string
                (Some(k), config::ValueKind::Array(arr)) => {
                    // Convert array elements to string and join with comma
                    let arr_str: Vec<String> = arr
                        .iter()
                        .map(|v| match &v.kind {
                            config::ValueKind::String(s) => s.clone(),
                            config::ValueKind::I64(n) => n.to_string(),
                            config::ValueKind::I128(n) => n.to_string(),
                            config::ValueKind::U64(n) => n.to_string(),
                            config::ValueKind::U128(n) => n.to_string(),
                            config::ValueKind::Float(n) => n.to_string(),
                            config::ValueKind::Boolean(b) => b.to_string(),
                            config::ValueKind::Table(_) => {
                                // For nested tables in arrays, use debug representation
                                format!("{:?}", v)
                            }
                            config::ValueKind::Array(_) => {
                                // For nested arrays, use debug representation
                                format!("{:?}", v)
                            }
                            config::ValueKind::Nil => {
                                // For nil values in arrays, use empty string
                                String::new()
                            }
                        })
                        .collect();
                    ps.put(k, arr_str.join(","));
                }
                // Nil type with prefix - skip null values
                (Some(_k), config::ValueKind::Nil) => {
                    // Skip null values - don't add to parameter scope
                }
                // Root level non-table types - should not occur in normal config, but handle gracefully
                (None, config::ValueKind::Boolean(_)) => {
                    // Root level boolean - skip (config root should be a table)
                }
                (None, config::ValueKind::I64(_)) => {
                    // Root level integer - skip (config root should be a table)
                }
                (None, config::ValueKind::I128(_)) => {
                    // Root level i128 - skip (config root should be a table)
                }
                (None, config::ValueKind::U64(_)) => {
                    // Root level u64 - skip (config root should be a table)
                }
                (None, config::ValueKind::U128(_)) => {
                    // Root level u128 - skip (config root should be a table)
                }
                (None, config::ValueKind::Float(_)) => {
                    // Root level float - skip (config root should be a table)
                }
                (None, config::ValueKind::String(_)) => {
                    // Root level string - skip (config root should be a table)
                }
                (None, config::ValueKind::Array(_)) => {
                    // Root level array - skip (config root should be a table)
                }
                (None, config::ValueKind::Nil) => {
                    // Root level nil - skip (config root should be a table)
                }
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
