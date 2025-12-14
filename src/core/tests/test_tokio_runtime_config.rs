//! 演示如何在测试中配置 tokio runtime 的线程数
//!
//! 方法 1: 使用 #[tokio::test] 宏的参数
//! 方法 2: 手动创建 Runtime
//! 方法 3: 使用环境变量

use hyperparameter::{get_param, with_params, GetOrElse};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::runtime::Builder;

static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

/// 方法 1: 使用 #[tokio::test] 宏的参数指定线程数
///
/// 语法: #[tokio::test(flavor = "multi_thread", worker_threads = N)]
///
/// - flavor = "multi_thread": 使用多线程运行时（默认是单线程）
/// - worker_threads = N: 指定工作线程数量
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_with_2_threads() {
    let count = Arc::new(AtomicUsize::new(0));

    // 创建多个并发任务来验证线程数
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let count = count.clone();
            tokio::spawn(async move {
                // 记录当前任务运行的线程
                let thread_id = std::thread::current().id();
                count.fetch_add(1, Ordering::SeqCst);

                with_params! {
                    @set task.id = i;
                    let val: i64 = get_param!(task.id, 0);
                    assert_eq!(val, i);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }

    assert_eq!(count.load(Ordering::SeqCst), 10);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_with_4_threads() {
    let count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let count = count.clone();
            tokio::spawn(async move {
                count.fetch_add(1, Ordering::SeqCst);

                with_params! {
                    @set task.id = i;
                    let val: i64 = get_param!(task.id, 0);
                    assert_eq!(val, i);
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }

    assert_eq!(count.load(Ordering::SeqCst), 20);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_with_8_threads() {
    let count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..50)
        .map(|i| {
            let count = count.clone();
            tokio::spawn(async move {
                count.fetch_add(1, Ordering::SeqCst);

                with_params! {
                    @set task.id = i;
                    let val: i64 = get_param!(task.id, 0);
                    assert_eq!(val, i);
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }

    assert_eq!(count.load(Ordering::SeqCst), 50);
}

/// 方法 2: 手动创建 Runtime 并配置线程数
///
/// 这种方式适合需要更精细控制的场景，比如在测试函数内部创建 runtime
#[test]
fn test_manual_runtime_with_threads() {
    // 创建指定线程数的 runtime
    let runtime = Builder::new_multi_thread()
        .worker_threads(4) // 设置 4 个工作线程
        .enable_all() // 启用所有功能（I/O, time, etc.）
        .build()
        .expect("Failed to create runtime");

    runtime.block_on(async {
        let count = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let count = count.clone();
                tokio::spawn(async move {
                    count.fetch_add(1, Ordering::SeqCst);

                    with_params! {
                        @set task.id = i;
                        let val: i64 = get_param!(task.id, 0);
                        assert_eq!(val, i);
                    }
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }

        assert_eq!(count.load(Ordering::SeqCst), 10);
    });
}

/// 方法 3: 使用环境变量控制（需要在运行时设置）
///
/// 可以通过设置 TOKIO_WORKER_THREADS 环境变量来控制
/// 但这种方式在 #[tokio::test] 中不太适用，更适合手动创建 runtime
#[test]
fn test_runtime_with_env_threads() {
    // 从环境变量读取线程数，如果没有设置则使用默认值
    let thread_count = std::env::var("TOKIO_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2); // 默认 2 个线程

    let runtime = Builder::new_multi_thread()
        .worker_threads(thread_count)
        .enable_all()
        .build()
        .expect("Failed to create runtime");

    runtime.block_on(async {
        let count = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let count = count.clone();
                tokio::spawn(async move {
                    count.fetch_add(1, Ordering::SeqCst);

                    with_params! {
                        @set task.id = i;
                        let val: i64 = get_param!(task.id, 0);
                        assert_eq!(val, i);
                    }
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }

        assert_eq!(count.load(Ordering::SeqCst), 10);
    });
}

/// 辅助宏：创建指定线程数的测试
///
/// 使用方式：
/// ```rust
/// tokio_test_with_threads!(4, async {
///     // 测试代码
/// });
/// ```
macro_rules! tokio_test_with_threads {
    ($threads:expr, $test:expr) => {
        #[tokio::test(flavor = "multi_thread", worker_threads = $threads)]
        async fn test() {
            $test.await
        }
    };
}

// 使用辅助宏创建测试
tokio_test_with_threads!(6, async {
    let count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..30)
        .map(|i| {
            let count = count.clone();
            tokio::spawn(async move {
                count.fetch_add(1, Ordering::SeqCst);

                with_params! {
                    @set task.id = i;
                    let val: i64 = get_param!(task.id, 0);
                    assert_eq!(val, i);
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }

    assert_eq!(count.load(Ordering::SeqCst), 30);
});
