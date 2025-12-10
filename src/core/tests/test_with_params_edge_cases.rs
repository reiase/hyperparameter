/// 测试 with_params! 宏的边界情况和常见陷阱
use hyperparameter::*;
use std::collections::HashMap;

#[test]
fn test_method_calls_named_get() {
    // 问题：map.get() 方法调用会被误识别为 get 指令
    let result = with_params! {
        @set config.value = 42;

        let mut map = HashMap::new();
        map.insert("key", 100);

        // 这里的 get 是 HashMap 的方法，不应被解析为指令
        let val = map.get("key").copied().unwrap_or(0);
        val
    };
    assert_eq!(result, 100);
}

#[test]
fn test_method_calls_named_set() {
    // 问题：自定义类型的 set 方法会被误识别
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
        config.set(200);  // 这应该调用 Config::set，不是指令
        let result = config.get();  // 同样，这是方法调用
        result
    };
    assert_eq!(result, 200);
}

#[test]
fn test_variables_named_get_or_set() {
    // 问题：变量名为 get/set 时会被误识别
    let result = with_params! {
        @set config.x = 10;

        let set = 50;  // 变量名叫 set
        let get = 30;  // 变量名叫 get

        set + get  // 这是普通的加法运算
    };
    assert_eq!(result, 80);
}

#[test]
fn test_function_calls_named_set_get() {
    // 问题：函数名为 set/get 时会被误识别
    fn set(x: i64) -> i64 {
        x * 2
    }
    fn get(x: i64) -> i64 {
        x + 10
    }

    let result = with_params! {
        @set param.value = 5;

        let a = set(20);  // 调用函数 set
        let b = get(15);  // 调用函数 get
        a + b
    };
    assert_eq!(result, 65); // 20*2 + 15+10 = 65
}

#[test]
fn test_params_in_macro_calls() {
    // 问题：宏调用中包含 set/get/params 关键字
    let result = with_params! {
        @set config.value = 100;

        let vec = vec![1, 2, 3];
        let subset = vec.iter().filter(|&&x| x > 1).collect::<Vec<_>>();

        // println! 等宏内部可能包含这些标识符
        println!("Debug: set={}, get={}", subset.len(), vec.len());

        subset.len()
    };
    assert_eq!(result, 2);
}

#[test]
fn test_trailing_semicolon_in_expressions() {
    // 边界情况：表达式末尾的分号处理
    let result = with_params! {
        @set val = 10;

        let x = 20;
        x + 5;  // 带分号的语句
        42      // 真正的返回值
    };
    assert_eq!(result, 42);
}

#[test]
fn test_nested_blocks_with_get_set() {
    // 嵌套块中的 get/set 方法调用
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
    // match 表达式中使用 get 方法
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
    // 闭包内使用 get/set 方法
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
    // 测试 @params 语法是否正常工作
    // 创建一个包含参数的 ParamScope
    let mut scope = ParamScope::default();
    scope.put("test.value", 42i64);

    // 使用 @params 语法来使用这个 scope
    let result = with_params! {
        @params scope;

        @get val = test.value or 0;
        val
    };
    assert_eq!(result, 42);

    // 测试 @params 和 params 都可以工作
    let mut scope2 = ParamScope::default();
    scope2.put("test.value", 100i64);
    let result2 = with_params! {
        params scope2;  // 不带 @ 的语法也应该工作

        @get val = test.value or 0;
        val
    };
    assert_eq!(result2, 100);
}

// 辅助函数
fn create_scope() -> ParamScope {
    ParamScope::capture()
}

fn create_another_scope() -> ParamScope {
    ParamScope::capture()
}
