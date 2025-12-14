/// Test edge cases and common pitfalls of the with_params! macro
use hyperparameter::*;
use std::collections::HashMap;

#[test]
fn test_method_calls_named_get() {
    // Issue: map.get() method calls may be mistakenly identified as get directives
    let result = with_params! {
        @set config.value = 42;

        let mut map = HashMap::new();
        map.insert("key", 100);

        // get here is a HashMap method, should not be parsed as a directive
        let val = map.get("key").copied().unwrap_or(0);
        val
    };
    assert_eq!(result, 100);
}

#[test]
fn test_method_calls_named_set() {
    // Issue: set methods of custom types may be mistakenly identified
    struct Config {
        value: i64,
    }

    impl Config {
        fn set(&mut self, val: i64) {
            self.value = val;
        }

        fn get(&self) -> i64 {
            self.value
        }
    }

    let result = with_params! {
        @set test.param = 1;

        let mut config = Config { value: 0 };
        config.set(200);  // This should call Config::set, not a directive
        let result = config.get();  // Similarly, this is a method call
        result
    };
    assert_eq!(result, 200);
}

#[test]
fn test_variables_named_get_or_set() {
    // Issue: variables named get/set may be mistakenly identified
    let result = with_params! {
        @set config.x = 10;

        let set = 50;  // Variable named set
        let get = 30;  // Variable named get

        set + get  // This is a normal addition operation
    };
    assert_eq!(result, 80);
}

#[test]
fn test_function_calls_named_set_get() {
    // Issue: functions named set/get may be mistakenly identified
    fn set(x: i64) -> i64 {
        x * 2
    }
    fn get(x: i64) -> i64 {
        x + 10
    }

    let result = with_params! {
        @set param.value = 5;

        let a = set(20);  // Call function set
        let b = get(15);  // Call function get
        a + b
    };
    assert_eq!(result, 65); // 20*2 + 15+10 = 65
}

#[test]
fn test_params_in_macro_calls() {
    // Issue: macro calls may contain set/get/params keywords
    let result = with_params! {
        @set config.value = 100;

        let vec = vec![1, 2, 3];
        let subset = vec.iter().filter(|&&x| x > 1).collect::<Vec<_>>();

        // Macros like println! may contain these identifiers internally
        println!("Debug: set={}, get={}", subset.len(), vec.len());

        subset.len()
    };
    assert_eq!(result, 2);
}

#[test]
fn test_trailing_semicolon_in_expressions() {
    // Edge case: handling semicolons at the end of expressions
    let result = with_params! {
        @set val = 10;

        let x = 20;
        x + 5;  // Statement with semicolon
        42      // The actual return value
    };
    assert_eq!(result, 42);
}

#[test]
fn test_nested_blocks_with_get_set() {
    // get/set method calls in nested blocks
    let result = with_params! {
        @set outer.value = 100;

        {
            let mut map = HashMap::new();
            map.insert("key", 50);

            if let Some(v) = map.get("key") {
                *v
            } else {
                0
            }
        }
    };
    assert_eq!(result, 50);
}

#[test]
fn test_match_expressions_with_get() {
    // Using get method in match expressions
    let result = with_params! {
        @set config.mode = "test".to_string();

        let mut map = HashMap::new();
        map.insert("mode", 42);

        match map.get("mode") {
            Some(v) => *v,
            None => 0,
        }
    };
    assert_eq!(result, 42);
}

#[test]
fn test_closure_with_get_set() {
    // Using get/set methods within closures
    let result = with_params! {
        @set param.x = 10;

        let mut map = HashMap::new();
        map.insert("a", 100);

        let closure = || {
            map.get("a").copied().unwrap_or(0)
        };

        closure()
    };
    assert_eq!(result, 100);
}

#[test]
fn test_at_params_syntax() {
    // Test if @params syntax works correctly
    // Create a ParamScope containing parameters
    let mut scope = ParamScope::default();
    scope.put("test.value", 42i64);

    // Use @params syntax to use this scope
    let result = with_params! {
        @params scope;

        @get val = test.value or 0;
        val
    };
    assert_eq!(result, 42);

    // Test that both @params and params work
    let mut scope2 = ParamScope::default();
    scope2.put("test.value", 100i64);
    let result2 = with_params! {
        params scope2;  // Syntax without @ should also work

        @get val = test.value or 0;
        val
    };
    assert_eq!(result2, 100);
}

// Helper functions
fn create_scope() -> ParamScope {
    ParamScope::capture()
}

fn create_another_scope() -> ParamScope {
    ParamScope::capture()
}
