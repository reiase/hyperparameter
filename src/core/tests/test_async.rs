use hyperparameter::{with_params, get_param, GetOrElse};

// Mock async functions for testing
async fn fetch_data() -> i64 {
    42
}

async fn fetch_with_param(_key: &str) -> i64 {
    let val: i64 = get_param!(test.key, 0);
    val + 1
}

async fn fetch_user() -> String {
    "user".to_string()
}

// ========== 异步检测测试 ==========
// 测试宏能否正确检测异步上下文并切换到异步模式

#[tokio::test]
async fn test_detects_explicit_await() {
    // Test: explicit .await should trigger async mode
    let result = with_params! {
        set test.value = 100;
        
        fetch_data().await  // Explicit await
    };
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_detects_async_function_calls() {
    // Test: calling an async function should trigger async mode
    let result = with_params! {
        set test.value = 1;
        
        fetch_data()  // No .await, but should be detected as async
    };
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_detects_async_blocks() {
    // Test: async blocks should trigger async mode
    let result = with_params! {
        set test.key = 50;
        
        async { 200 }  // Should be detected and auto-awaited
    };
    assert_eq!(result, 200);
}

#[tokio::test]
async fn test_detects_by_function_name_pattern() {
    // Test: function names like "fetch" should trigger async mode (heuristic)
    let result = with_params! {
        set user.name = "test";
        
        fetch_user()  // Should be detected as async by name pattern
    };
    assert_eq!(result, "user");
}

#[tokio::test]
async fn test_does_not_detect_sync_code() {
    // Test: sync code should not be converted to async
    let result = with_params! {
        set test.value = 1;
        
        let x = 10;
        x + 1  // Sync expression - should stay sync
    };
    assert_eq!(result, 11);
}

// ========== 自动 await 测试 ==========
// 测试宏能否自动插入 .await

#[tokio::test]
async fn test_auto_awaits_async_function_calls() {
    // Test that async functions are automatically awaited
    let result = with_params! {
        set test.key = 10;
        
        fetch_data()  // Should be auto-awaited
    };
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_auto_awaits_with_parameters() {
    // Test auto-await with function parameters
    let result = with_params! {
        set test.key = 20;
        
        fetch_with_param("test")  // Should be auto-awaited
    };
    assert_eq!(result, 21);
}

#[tokio::test]
async fn test_auto_awaits_async_closures() {
    // Test: async closures should be auto-awaited
    let result = with_params! {
        set test.key = 50;
        
        async { 200 }  // Should be auto-awaited
    };
    assert_eq!(result, 200);
}

#[tokio::test]
async fn test_explicit_await_takes_precedence() {
    // Test: explicit .await should work and not be duplicated
    let result = with_params! {
        set test.key = 30;
        
        fetch_data().await  // Explicit await - should not add another
    };
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_does_not_await_join_handle() {
    // Test: JoinHandle should NOT be auto-awaited (user might want the handle)
    let handle = with_params! {
        set test.key = 40;
        
        tokio::spawn(async { 100 })  // Should NOT be auto-awaited
    };
    let result = handle.await.unwrap();
    assert_eq!(result, 100);
}

// ========== 边界情况测试 ==========

#[tokio::test]
async fn test_nested_async_with_params() {
    // Test: nested with_params in async context
    let result = with_params! {
        set outer.value = 1;
        
        with_params! {
            set inner.value = 2;
            
            async {
                let outer_val: i64 = get_param!(outer.value, 0);
                let inner_val: i64 = get_param!(inner.value, 0);
                outer_val + inner_val
            }
        }
    };
    assert_eq!(result, 3);
}

#[tokio::test]
async fn test_async_with_intermediate_await() {
    // Test: async context with intermediate explicit await
    // Only the last expression is auto-awaited, intermediate calls need explicit await
    let result = with_params! {
        set config.base = 10;
        
        let base: i64 = get_param!(config.base, 0);
        let async_val = fetch_data().await;  // Intermediate call needs explicit await
        base + async_val  // Last expression (sync)
    };
    assert_eq!(result, 52);
}
