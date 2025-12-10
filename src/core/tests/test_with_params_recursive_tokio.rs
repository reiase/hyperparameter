//! 测试 with_params 在 tokio runtime 上的随机递归深度场景
//!
//! 本测试文件包含 100+ 个测试用例，验证：
//! 1. 随机递归深度的嵌套 with_params 正确性
//! 2. 参数作用域的正确进入和退出
//! 3. 在异步上下文中的参数隔离
//! 4. 并发场景下的正确性

use hyperparameter::{get_param, with_current_storage, with_params, GetOrElse};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{sleep, Duration};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 生成测试 ID
fn next_test_id() -> u64 {
    TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 辅助函数：使用动态 key 获取参数
fn get_param_dynamic<T>(key: &str, default: T) -> T
where
    T: Into<hyperparameter::Value>
        + TryFrom<hyperparameter::Value>
        + for<'a> TryFrom<&'a hyperparameter::Value>,
{
    with_current_storage(|ts| ts.get_or_else(key, default))
}

/// 递归设置和获取参数，验证作用域正确性
/// 逻辑：每层设置 param = depth，进入下一层后校验获取到的 param 是否为上一层的 depth
fn recursive_test_inner(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = i64> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            // 到达最大深度，读取参数（应该是上一层的 depth，即 max_depth - 1）
            let val: i64 = get_param_dynamic("test_key", -1);
            if max_depth > 0 {
                assert_eq!(
                    val,
                    (max_depth - 1) as i64,
                    "最大深度时参数应该是 {}",
                    max_depth - 1
                );
            } else {
                // max_depth=0 时，没有参数被设置，应该返回默认值 -1
                assert_eq!(val, -1, "max_depth=0 时应该返回默认值 -1");
            }
            val
        } else {
            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                let prev_val: i64 = get_param_dynamic("test_key", -1);
                if depth > 0 {
                    // depth > 0 时，应该能读取到上一层的值（depth-1）
                    assert_eq!(prev_val, (depth - 1) as i64, "深度 {} 的传入参数值应该是上一层的 {}", depth, depth - 1);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值 -1
                    assert_eq!(prev_val, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key = depth as i64;

                // 递归调用到下一层
                let result = recursive_test_inner(depth + 1, max_depth, test_id).await;

                // 验证当前层级的参数仍然正确（应该是当前 depth）
                let current_val: i64 = get_param_dynamic("test_key", -1);
                assert_eq!(current_val, depth as i64, "深度 {} 的参数值应该是 {}", depth, depth);

                // 返回下一层的结果（下一层会验证它读取到的参数是当前层的 depth）
                result
            }
        }
    })
}

/// 测试用例 1-10: 基础递归测试
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_recursive_basic_1() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 1, test_id).await;
    assert_eq!(result, 0); // max_depth=1, 读取到的应该是 0
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_recursive_basic_2() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 2, test_id).await;
    assert_eq!(result, 1); // max_depth=2, 读取到的应该是 1
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_recursive_basic_3() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 3, test_id).await;
    assert_eq!(result, 2); // max_depth=3, 读取到的应该是 2
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_recursive_basic_4() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 4, test_id).await;
    assert_eq!(result, 3); // max_depth=4, 读取到的应该是 3
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_recursive_basic_5() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 5, test_id).await;
    assert_eq!(result, 4); // max_depth=5, 读取到的应该是 4
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_recursive_basic_6() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 6, test_id).await;
    assert_eq!(result, 5); // max_depth=6, 读取到的应该是 5
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_recursive_basic_7() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 7, test_id).await;
    assert_eq!(result, 6); // max_depth=7, 读取到的应该是 6
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_recursive_basic_8() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 8, test_id).await;
    assert_eq!(result, 7); // max_depth=8, 读取到的应该是 7
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_recursive_basic_9() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 9, test_id).await;
    assert_eq!(result, 8); // max_depth=9, 读取到的应该是 8
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_recursive_basic_10() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 10, test_id).await;
    assert_eq!(result, 9); // max_depth=10, 读取到的应该是 9
}

/// 测试用例 11-20: 随机深度测试（使用固定种子）
fn random_depth(seed: u64, max: usize) -> usize {
    // 简单的线性同余生成器
    let mut x = seed;
    x = x.wrapping_mul(1103515245).wrapping_add(12345);
    (x as usize) % max + 1
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_random_depth_1() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 5);
    let result = recursive_test_inner(0, depth, test_id).await;
    // 验证结果：max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_random_depth_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let result = recursive_test_inner(0, depth, test_id).await;
    let expected = if depth > 1 {
        (depth - 2) * (depth - 1) / 2
    } else {
        0
    };
    assert_eq!(result, expected as i64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_random_depth_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let result = recursive_test_inner(0, depth, test_id).await;
    let expected = if depth > 1 {
        (depth - 2) * (depth - 1) / 2
    } else {
        0
    };
    assert_eq!(result, expected as i64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_random_depth_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let result = recursive_test_inner(0, depth, test_id).await;
    let expected = if depth > 1 {
        (depth - 2) * (depth - 1) / 2
    } else {
        0
    };
    assert_eq!(result, expected as i64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_random_depth_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let result = recursive_test_inner(0, depth, test_id).await;
    let expected = if depth > 1 {
        (depth - 2) * (depth - 1) / 2
    } else {
        0
    };
    assert_eq!(result, expected as i64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_random_depth_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let result = recursive_test_inner(0, depth, test_id).await;
    let expected = if depth > 1 {
        (depth - 2) * (depth - 1) / 2
    } else {
        0
    };
    assert_eq!(result, expected as i64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_random_depth_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let result = recursive_test_inner(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_random_depth_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let result = recursive_test_inner(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_random_depth_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let result = recursive_test_inner(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_random_depth_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 50);
    let result = recursive_test_inner(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

/// 测试用例 21-30: 参数覆盖测试
/// 逻辑：每层设置 param = depth，收集所有层级的参数值（从最深到最浅）
fn recursive_override_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<i64>> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            // 到达最大深度，不读取参数，直接返回空列表
            vec![]
        } else {
            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                let prev_val: i64 = get_param_dynamic("test_key_override", -1);
                if depth > 0 {
                    // depth > 0 时，应该能读取到上一层的值（depth-1）
                    assert_eq!(prev_val, (depth - 1) as i64, "深度 {} 的传入参数值应该是上一层的 {}", depth, depth - 1);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值 -1
                    assert_eq!(prev_val, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key_override = depth as i64;

                let mut results = recursive_override_test(depth + 1, max_depth, test_id).await;

                // 验证当前层级的参数仍然正确
                let current_val: i64 = get_param_dynamic("test_key_override", -1);
                assert_eq!(current_val, depth as i64, "深度 {} 的参数应该是 {}", depth, depth);

                // 从最深到最浅收集参数值
                results.push(current_val);
                results
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_override_1() {
    let test_id = next_test_id();
    let results = recursive_override_test(0, 3, test_id).await;
    // 从最深到最浅：max_depth=3 时读取到 2，然后依次是 1, 0
    assert_eq!(results, vec![2, 1, 0]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_override_2() {
    let test_id = next_test_id();
    let results = recursive_override_test(0, 5, test_id).await;
    // 从最深到最浅：max_depth=5 时读取到 4，然后依次是 3, 2, 1, 0
    assert_eq!(results, vec![4, 3, 2, 1, 0]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_override_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let results = recursive_override_test(0, depth, test_id).await;
    // 验证结果从最大深度到 0 递减（从最深到最浅）
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(
            val,
            (depth - 1 - i) as i64,
            "位置 {} 的值应该是 {}",
            i,
            depth - 1 - i
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_override_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_override_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_override_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_override_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_override_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_override_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_override_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let results = recursive_override_test(0, depth, test_id).await;
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

/// 测试用例 31-40: 多参数递归测试
/// 逻辑：每层设置多个参数为 depth 相关的值，进入下一层后校验获取到的参数是否为上一层的值
fn recursive_multi_param_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = (i64, f64, String)> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            // 到达最大深度，读取参数（应该是上一层的值，即 max_depth - 1）
            let prev_depth = (max_depth - 1) as i64;
            let int_val: i64 = get_param_dynamic("test_key_override_int", -1);
            let float_val: f64 = get_param_dynamic("test_key_override_float", -1.0);
            let str_val: String = get_param_dynamic("test_key_override_str", "".to_string());

            assert_eq!(
                int_val, prev_depth,
                "最大深度时 int 参数应该是 {}",
                prev_depth
            );
            assert!(
                (float_val - prev_depth as f64 * 1.5).abs() < 1e-10,
                "最大深度时 float 参数应该是 {}",
                prev_depth as f64 * 1.5
            );
            assert_eq!(
                str_val,
                format!("depth_{}", max_depth - 1),
                "最大深度时 str 参数应该是 depth_{}",
                max_depth - 1
            );

            (int_val, float_val, str_val)
        } else {
            let int_val = depth as i64;
            let float_val = depth as f64 * 1.5;
            let str_val = format!("depth_{}", depth);

            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                if depth > 0 {
                    let prev_int: i64 = get_param_dynamic("test_key_override_int", -1);
                    let prev_float: f64 = get_param_dynamic("test_key_override_float", -1.0);
                    let prev_str: String = get_param_dynamic("test_key_override_str", "".to_string());

                    let expected_prev_int = (depth - 1) as i64;
                    let expected_prev_float = (depth - 1) as f64 * 1.5;
                    let expected_prev_str = format!("depth_{}", depth - 1);

                    assert_eq!(prev_int, expected_prev_int, "深度 {} 的传入 int 参数值应该是上一层的 {}", depth, expected_prev_int);
                    assert!((prev_float - expected_prev_float).abs() < 1e-10, "深度 {} 的传入 float 参数值应该是上一层的 {}", depth, expected_prev_float);
                    assert_eq!(prev_str, expected_prev_str, "深度 {} 的传入 str 参数值应该是上一层的 {}", depth, expected_prev_str);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值
                    let prev_int: i64 = get_param_dynamic("test_key_override_int", -1);
                    assert_eq!(prev_int, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key_override_int = int_val;
                @set test_key_override_float = float_val;
                @set test_key_override_str = str_val.clone();

                let (inner_int, inner_float, inner_str) =
                    recursive_multi_param_test(depth + 1, max_depth, test_id).await;

                // 验证当前层级的参数仍然正确
                let current_int: i64 = get_param_dynamic("test_key_override_int", -1);
                let current_float: f64 = get_param_dynamic("test_key_override_float", -1.0);
                let current_str: String = get_param_dynamic("test_key_override_str", "".to_string());

                assert_eq!(current_int, int_val, "深度 {} 的 int 参数应该是 {}", depth, int_val);
                assert!((current_float - float_val).abs() < 1e-10, "深度 {} 的 float 参数应该是 {}", depth, float_val);
                assert_eq!(current_str, str_val, "深度 {} 的 str 参数应该是 {}", depth, str_val);

                // 返回下一层的结果（下一层会验证它读取到的参数是当前层的值）
                (inner_int, inner_float, inner_str)
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_multi_param_1() {
    let test_id = next_test_id();
    let (int_val, float_val, str_result) = recursive_multi_param_test(0, 3, test_id).await;
    // max_depth=3 时，读取到的应该是 depth=2 的值
    assert_eq!(int_val, 2);
    assert!((float_val - 3.0).abs() < 1e-10); // 2 * 1.5 = 3.0
    assert_eq!(str_result, "depth_2");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_multi_param_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let (int_val, _, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(int_val, expected_int);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multi_param_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let (int_val, float_val, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 1.5
    } else {
        -1.0
    };
    assert_eq!(int_val, expected_int);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_multi_param_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let (int_sum, _, _) = recursive_multi_param_test(0, depth, test_id).await;
    let expected_int = (0..depth).sum::<usize>() as i64;
    assert_eq!(int_sum, expected_int);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_multi_param_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let (int_val, float_val, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 1.5
    } else {
        -1.0
    };
    assert_eq!(int_val, expected_int);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_multi_param_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let (int_sum, _, _) = recursive_multi_param_test(0, depth, test_id).await;
    let expected_int = (0..depth).sum::<usize>() as i64;
    assert_eq!(int_sum, expected_int);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multi_param_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let (int_val, float_val, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 1.5
    } else {
        -1.0
    };
    assert_eq!(int_val, expected_int);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multi_param_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let (int_sum, _, _) = recursive_multi_param_test(0, depth, test_id).await;
    let expected_int = (0..depth).sum::<usize>() as i64;
    assert_eq!(int_sum, expected_int);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_multi_param_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let (int_val, float_val, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 1.5
    } else {
        -1.0
    };
    assert_eq!(int_val, expected_int);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_multi_param_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 50);
    let (int_val, _, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(int_val, expected_int);
}

/// 测试用例 41-50: 异步操作中的递归测试
/// 逻辑：每层设置 param = depth，进入下一层后校验获取到的 param 是否为上一层的 depth
fn recursive_async_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = i64> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            sleep(Duration::from_millis(1)).await;
            // 到达最大深度，读取参数（应该是上一层的 depth，即 max_depth - 1）
            let val: i64 = get_param_dynamic("test_key_async", -1);
            assert_eq!(
                val,
                (max_depth - 1) as i64,
                "最大深度时参数应该是 {}",
                max_depth - 1
            );
            val
        } else {
            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                let prev_val: i64 = get_param_dynamic("test_key_async", -1);
                if depth > 0 {
                    // depth > 0 时，应该能读取到上一层的值（depth-1）
                    assert_eq!(prev_val, (depth - 1) as i64, "深度 {} 的传入参数值应该是上一层的 {}", depth, depth - 1);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值 -1
                    assert_eq!(prev_val, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key_async = depth as i64;

                sleep(Duration::from_millis(1)).await;

                let result = recursive_async_test(depth + 1, max_depth, test_id).await;

                sleep(Duration::from_millis(1)).await;

                // 验证当前层级的参数仍然正确
                let current_val: i64 = get_param_dynamic("test_key_async", -1);
                assert_eq!(current_val, depth as i64, "异步深度 {} 的参数值应该是 {}", depth, depth);

                // 返回下一层的结果
                result
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_async_recursive_1() {
    let test_id = next_test_id();
    let result = recursive_async_test(0, 3, test_id).await;
    assert_eq!(result, 2); // max_depth=3, 读取到的应该是 2
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_async_recursive_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_async_recursive_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_async_recursive_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_async_recursive_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_async_recursive_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_async_recursive_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_recursive_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_async_recursive_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_async_recursive_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 50);
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

/// 测试用例 51-60: 并发递归测试
/// 逻辑：每层设置 param = depth * 100 + task_id，进入下一层后校验获取到的 param 是否为上一层的值
fn concurrent_recursive_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
    task_id: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = i64> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            // 到达最大深度，读取参数（应该是上一层的值）
            let prev_value = ((max_depth - 1) * 100 + task_id) as i64;
            let val: i64 = get_param_dynamic("test_key_concurrent", -1);
            assert_eq!(val, prev_value, "最大深度时参数应该是 {}", prev_value);
            val
        } else {
            let value = (depth * 100 + task_id) as i64;

            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                let prev_val: i64 = get_param_dynamic("test_key_concurrent", -1);
                if depth > 0 {
                    // depth > 0 时，应该能读取到上一层的值
                    let expected_prev_value = ((depth - 1) * 100 + task_id) as i64;
                    assert_eq!(prev_val, expected_prev_value, "任务 {} 深度 {} 的传入参数值应该是上一层的 {}", task_id, depth, expected_prev_value);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值 -1
                    assert_eq!(prev_val, -1, "任务 {} 深度 0 时应该读取不到参数，返回默认值 -1", task_id);
                }

                @set test_key_concurrent = value;

                let result = concurrent_recursive_test(depth + 1, max_depth, test_id, task_id).await;

                // 验证当前层级的参数仍然正确
                let current_val: i64 = get_param_dynamic("test_key_concurrent", -1);
                assert_eq!(current_val, value, "任务 {} 深度 {} 的参数值应该是 {}", task_id, depth, value);

                // 返回下一层的结果
                result
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_concurrent_recursive_1() {
    let test_id = next_test_id();
    let depth = 5;
    let handles: Vec<_> = (0..5)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // 每个任务的结果应该不同（因为 task_id 不同）
    for (i, &result) in results.iter().enumerate() {
        assert!(result > 0, "任务 {} 的结果应该大于 0", i);
    }

    // 验证所有任务的结果都不同
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(results[i], results[j], "不同任务的结果应该不同");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_concurrent_recursive_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let handles: Vec<_> = (0..10)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 10);
    for (i, &result) in results.iter().enumerate() {
        assert!(result > 0, "任务 {} 的结果应该大于 0", i);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_concurrent_recursive_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let handles: Vec<_> = (0..15)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 15);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_concurrent_recursive_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let handles: Vec<_> = (0..20)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 20);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_recursive_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let handles: Vec<_> = (0..25)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 25);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_recursive_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let handles: Vec<_> = (0..30)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 30);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_concurrent_recursive_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 12);
    let handles: Vec<_> = (0..35)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 35);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_concurrent_recursive_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let handles: Vec<_> = (0..40)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 40);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_concurrent_recursive_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 18);
    let handles: Vec<_> = (0..45)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 45);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_recursive_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let handles: Vec<_> = (0..50)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 50);
}

/// 测试用例 61-70: 混合场景测试
/// 逻辑：每层设置参数为 depth 相关的值，进入下一层后校验获取到的参数是否为上一层的值
fn mixed_scenario_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = (i64, f64)> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            sleep(Duration::from_nanos(100)).await;
            // 到达最大深度，读取参数（应该是上一层的值，即 max_depth - 1）
            let prev_depth = max_depth - 1;
            let int_val: i64 = get_param_dynamic("test_key_mixed_int", -1);
            let float_val: f64 = get_param_dynamic("test_key_mixed_float", -1.0);

            let expected_int = prev_depth as i64 * 2;
            let expected_float = prev_depth as f64 * 3.14;
            assert_eq!(
                int_val, expected_int,
                "最大深度时 int 参数应该是 {}",
                expected_int
            );
            assert!(
                (float_val - expected_float).abs() < 1e-10,
                "最大深度时 float 参数应该是 {}",
                expected_float
            );

            (int_val, float_val)
        } else {
            let int_val = depth as i64 * 2;
            let float_val = depth as f64 * 3.14;

            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                if depth > 0 {
                    let prev_int: i64 = get_param_dynamic("test_key_mixed_int", -1);
                    let prev_float: f64 = get_param_dynamic("test_key_mixed_float", -1.0);

                    let expected_prev_int = (depth - 1) as i64 * 2;
                    let expected_prev_float = (depth - 1) as f64 * 3.14;

                    assert_eq!(prev_int, expected_prev_int, "深度 {} 的传入 int 参数值应该是上一层的 {}", depth, expected_prev_int);
                    assert!((prev_float - expected_prev_float).abs() < 1e-10, "深度 {} 的传入 float 参数值应该是上一层的 {}", depth, expected_prev_float);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值
                    let prev_int: i64 = get_param_dynamic("test_key_mixed_int", -1);
                    assert_eq!(prev_int, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key_mixed_int = int_val;
                @set test_key_mixed_float = float_val;

                sleep(Duration::from_nanos(100)).await;

                let (inner_int, inner_float) = mixed_scenario_test(depth + 1, max_depth, test_id).await;

                sleep(Duration::from_nanos(100)).await;

                // 验证当前层级的参数仍然正确
                let current_int: i64 = get_param_dynamic("test_key_mixed_int", -1);
                let current_float: f64 = get_param_dynamic("test_key_mixed_float", -1.0);

                assert_eq!(current_int, int_val, "深度 {} 的 int 参数应该是 {}", depth, int_val);
                assert!((current_float - float_val).abs() < 1e-10, "深度 {} 的 float 参数应该是 {}", depth, float_val);

                // 返回下一层的结果
                (inner_int, inner_float)
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_mixed_scenario_1() {
    let test_id = next_test_id();
    let (int_val, float_val) = mixed_scenario_test(0, 5, test_id).await;
    // max_depth=5 时，读取到的应该是 depth=4 的值
    assert_eq!(int_val, 8); // 4 * 2 = 8
    assert!((float_val - 12.56).abs() < 1e-5); // 4 * 3.14 = 12.56
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_mixed_scenario_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_mixed_scenario_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mixed_scenario_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_mixed_scenario_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_mixed_scenario_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_mixed_scenario_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mixed_scenario_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mixed_scenario_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_mixed_scenario_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 50);
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

/// 测试用例 71-80: 深度嵌套恢复测试
/// 逻辑：每层设置 param = depth，收集所有层级的参数值，验证作用域恢复
fn deep_nested_restore_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<i64>> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            vec![]
        } else {
            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                let prev_val: i64 = get_param_dynamic("test_key_restore", -1);
                if depth > 0 {
                    // depth > 0 时，应该能读取到上一层的值（depth-1）
                    assert_eq!(prev_val, (depth - 1) as i64, "深度 {} 的传入参数值应该是上一层的 {}", depth, depth - 1);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值 -1
                    assert_eq!(prev_val, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key_restore = depth as i64;

                let mut inner_results = deep_nested_restore_test(depth + 1, max_depth, test_id).await;

                // 在退出作用域前验证当前层级的参数仍然正确
                let before_exit: i64 = get_param_dynamic("test_key_restore", -1);
                assert_eq!(before_exit, depth as i64, "深度 {} 的参数应该是 {}", depth, depth);

                inner_results.push(before_exit);
                inner_results
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_deep_restore_1() {
    let test_id = next_test_id();
    let results = deep_nested_restore_test(0, 5, test_id).await;
    assert_eq!(results, vec![4, 3, 2, 1, 0]);

    // 验证作用域退出后参数不存在
    let val: i64 = get_param_dynamic("test_key_restore", -1);
    assert_eq!(val, -1, "作用域退出后参数应该不存在");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_deep_restore_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_deep_restore_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_deep_restore_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_deep_restore_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_deep_restore_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_deep_restore_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_deep_restore_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_deep_restore_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_deep_restore_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 50);
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

/// 测试用例 81-90: 复杂表达式测试
/// 逻辑：每层设置 base = depth+1, mult = depth+2，进入下一层后校验获取到的参数是否为上一层的值
fn complex_expression_test(
    depth: usize,
    max_depth: usize,
    test_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = i64> + Send>> {
    Box::pin(async move {
        if depth >= max_depth {
            // 到达最大深度，读取参数（应该是上一层的值，即 max_depth - 1）
            let prev_depth = max_depth - 1;
            let base: i64 = get_param_dynamic("test_key_base", -1);
            let mult: i64 = get_param_dynamic("test_key_mult", -1);

            let expected_base = (prev_depth + 1) as i64;
            let expected_mult = (prev_depth + 2) as i64;
            assert_eq!(
                base, expected_base,
                "最大深度时 base 应该是 {}",
                expected_base
            );
            assert_eq!(
                mult, expected_mult,
                "最大深度时 mult 应该是 {}",
                expected_mult
            );

            base * mult
        } else {
            let base = (depth + 1) as i64;
            let mult = (depth + 2) as i64;

            with_params! {
                // 在设置参数之前，检查是否能读取到上一层的参数值
                if depth > 0 {
                    let prev_base: i64 = get_param_dynamic("test_key_base", -1);
                    let prev_mult: i64 = get_param_dynamic("test_key_mult", -1);

                    let expected_prev_base = depth as i64; // (depth-1)+1 = depth
                    let expected_prev_mult = (depth + 1) as i64; // (depth-1)+2 = depth+1

                    assert_eq!(prev_base, expected_prev_base, "深度 {} 的传入 base 参数值应该是上一层的 {}", depth, expected_prev_base);
                    assert_eq!(prev_mult, expected_prev_mult, "深度 {} 的传入 mult 参数值应该是上一层的 {}", depth, expected_prev_mult);
                } else {
                    // depth = 0 时，没有上一层，应该返回默认值
                    let prev_base: i64 = get_param_dynamic("test_key_base", -1);
                    assert_eq!(prev_base, -1, "深度 0 时应该读取不到参数，返回默认值 -1");
                }

                @set test_key_base = base;
                @set test_key_mult = mult;

                let inner_result = complex_expression_test(depth + 1, max_depth, test_id).await;

                // 验证当前层级的参数仍然正确
                let current_base: i64 = get_param_dynamic("test_key_base", -1);
                let current_mult: i64 = get_param_dynamic("test_key_mult", -1);

                assert_eq!(current_base, base, "深度 {} 的 base 应该是 {}", depth, base);
                assert_eq!(current_mult, mult, "深度 {} 的 mult 应该是 {}", depth, mult);

                // 返回下一层的结果
                inner_result
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_complex_expr_1() {
    let test_id = next_test_id();
    let result = complex_expression_test(0, 3, test_id).await;
    // max_depth=3 时，读取到的应该是 depth=2 的值：base=3, mult=4, 所以 3*4=12
    assert_eq!(result, 12);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_complex_expr_2() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 10);
    let result = complex_expression_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值：base=depth, mult=depth+1
    if depth > 0 {
        let expected = (depth as i64) * ((depth + 1) as i64);
        assert_eq!(result, expected, "结果应该是 {}", expected);
    } else {
        assert_eq!(result, 1); // depth=0 时，base=1, mult=1
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_complex_expr_3() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 15);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_complex_expr_4() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 20);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_complex_expr_5() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 25);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_complex_expr_6() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 30);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_complex_expr_7() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 35);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_complex_expr_8() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 40);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_complex_expr_9() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 45);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_complex_expr_10() {
    let test_id = next_test_id();
    let depth = random_depth(test_id, 50);
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

/// 测试用例 91-100: 边界情况测试
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_edge_case_single_level() {
    let test_id = next_test_id();
    let result = with_params! {
        @set test.edge = 42;
        get_param!(test.edge, 0i64)
    };
    assert_eq!(result, 42);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_edge_case_zero_depth() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 0, test_id).await;
    // max_depth=0 时，没有参数被设置，应该返回默认值 -1
    assert_eq!(result, -1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_edge_case_one_depth() {
    let test_id = next_test_id();
    let result = recursive_test_inner(0, 1, test_id).await;
    assert_eq!(result, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_edge_case_empty_params() {
    let test_id = next_test_id();
    let result = with_params! {
        let x: i64 = get_param!(nonexistent.key, 100);
        x
    };
    assert_eq!(result, 100);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_edge_case_nested_empty() {
    let test_id = next_test_id();
    let result = with_params! {
        with_params! {
            with_params! {
                let x: i64 = get_param!(still.nonexistent, 200);
                x
            }
        }
    };
    assert_eq!(result, 200);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_edge_case_rapid_nesting() {
    let test_id = next_test_id();
    let result = with_params! {
        @set a = 1;
        with_params! {
            @set a = 2;
            with_params! {
                @set a = 3;
                with_params! {
                    @set a = 4;
                    let x: i64 = get_param!(a, 0);
                    x
                }
            }
        }
    };
    assert_eq!(result, 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_edge_case_rapid_unnesting() {
    let test_id = next_test_id();
    let (v1, v2, v3, v4) = with_params! {
        @set a = 1;
        let v1: i64 = get_param!(a, 0);
        with_params! {
            @set a = 2;
            let v2: i64 = get_param!(a, 0);
            with_params! {
                @set a = 3;
                let v3: i64 = get_param!(a, 0);
                with_params! {
                    @set a = 4;
                    let v4: i64 = get_param!(a, 0);
                    (v1, v2, v3, v4)
                }
            }
        }
    };
    assert_eq!(v1, 1);
    assert_eq!(v2, 2);
    assert_eq!(v3, 3);
    assert_eq!(v4, 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_edge_case_async_yield() {
    let test_id = next_test_id();
    let result = with_params! {
        @set test.yield_val = 50;
        tokio::task::yield_now().await;
        let x: i64 = get_param!(test.yield_val, 0);
        x
    };
    assert_eq!(result, 50);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_edge_case_many_params() {
    let test_id = next_test_id();
    let depth = 20;
    let result = with_params! {
        // 设置多个参数
        @set p1 = 1;
        @set p2 = 2;
        @set p3 = 3;
        @set p4 = 4;
        @set p5 = 5;

        with_params! {
            @set p1 = 10;
            @set p2 = 20;

            let v1: i64 = get_param!(p1, 0);
            let v2: i64 = get_param!(p2, 0);
            let v3: i64 = get_param!(p3, 0);
            let v4: i64 = get_param!(p4, 0);
            let v5: i64 = get_param!(p5, 0);

            v1 + v2 + v3 + v4 + v5
        }
    };
    assert_eq!(result, 10 + 20 + 3 + 4 + 5);
}

/// 测试用例 101-110: 额外压力测试
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_stress_deep_recursion_1() {
    let test_id = next_test_id();
    let depth = 100;
    let result = recursive_test_inner(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_stress_deep_recursion_2() {
    let test_id = next_test_id();
    let depth = 150;
    let result = recursive_test_inner(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 7)]
async fn test_stress_concurrent_deep() {
    let test_id = next_test_id();
    let depth = 30;
    let handles: Vec<_> = (0..20)
        .map(|task_id| tokio::spawn(concurrent_recursive_test(0, depth, test_id, task_id)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 20);
    for result in results {
        assert!(result > 0);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stress_mixed_deep() {
    let test_id = next_test_id();
    let depth = 40;
    let (int_val, float_val) = mixed_scenario_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1 的值
    let expected_int = if depth > 0 { (depth - 1) * 2 } else { 0 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 3.14
    } else {
        0.0
    };
    assert_eq!(int_val, expected_int as i64);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stress_override_deep() {
    let test_id = next_test_id();
    let depth = 50;
    let results = recursive_override_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
    for (i, &val) in results.iter().enumerate() {
        assert_eq!(val, (depth - 1 - i) as i64);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_stress_multi_param_deep() {
    let test_id = next_test_id();
    let depth = 60;
    let (int_val, float_val, _) = recursive_multi_param_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected_int = if depth > 0 { (depth - 1) as i64 } else { -1 };
    let expected_float = if depth > 0 {
        (depth - 1) as f64 * 1.5
    } else {
        -1.0
    };
    assert_eq!(int_val, expected_int);
    assert!((float_val - expected_float).abs() < 1e-5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_stress_async_deep() {
    let test_id = next_test_id();
    let depth = 70;
    let result = recursive_async_test(0, depth, test_id).await;
    // max_depth=depth 时，读取到的应该是 depth-1
    let expected = if depth > 0 { (depth - 1) as i64 } else { -1 };
    assert_eq!(result, expected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_stress_complex_deep() {
    let test_id = next_test_id();
    let depth = 80;
    let result = complex_expression_test(0, depth, test_id).await;
    assert!(result > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn test_stress_restore_deep() {
    let test_id = next_test_id();
    let depth = 90;
    let results = deep_nested_restore_test(0, depth, test_id).await;
    assert_eq!(results.len(), depth);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_stress_all_together() {
    let test_id = next_test_id();
    let depth = 25;

    // 组合所有测试场景
    let handles: Vec<_> = (0..10)
        .map(|task_id| {
            let tid = test_id;
            tokio::spawn(async move {
                let r1 = concurrent_recursive_test(0, depth, tid, task_id).await;
                let r2 = recursive_async_test(0, depth / 2, tid).await;
                let (r3, _) = mixed_scenario_test(0, depth / 3, tid).await;
                (r1, r2, r3)
            })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 10);
    for (r1, r2, r3) in results {
        assert!(r1 > 0);
        assert!(r2 >= 0);
        assert!(r3 > 0);
    }
}
