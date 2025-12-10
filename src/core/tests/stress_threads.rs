use std::thread;
use std::time::{Duration, Instant};

use hyperparameter::*;

/// Long-running multithread stress test; ignored by default.
#[test]
#[ignore = "30s stress test"]
fn stress_param_scope_multithread_30s() {
    // Seed a global value so spawned threads see the frozen snapshot.
    with_params! {
        @set baseline.seed = 7i64;
        frozen();
    }

    let worker_count = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let deadline = Instant::now() + Duration::from_secs(30);

    let mut handles = Vec::with_capacity(worker_count);
    for worker_id in 0..worker_count {
        handles.push(thread::spawn(move || {
            let mut iter: i64 = 0;
            while Instant::now() < deadline {
                with_params! {
                    // per-iteration writes
                    @set worker.id = worker_id as i64;
                    @set worker.iter = iter;

                    // read baseline propagated via frozen()
                    @get seed = baseline.seed or 0i64;
                    // read back what we just set
                    @get wid = worker.id or -1i64;
                    @get witer = worker.iter or -1i64;

                    assert_eq!(seed, 7);
                    assert_eq!(wid, worker_id as i64);
                    assert_eq!(witer, iter);
                };
                iter += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().expect("worker panicked");
    }
}
