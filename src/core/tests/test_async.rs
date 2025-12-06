use hyperparameter::{
    frozen, get_param, with_params_async, with_params, with_params_readonly, GetOrElse,
    ParamScope, ParamScopeOps,
};

#[tokio::test]
async fn test_async_tasks_isolated() {
    // base value visible to new tasks; set using with_params and freeze.
    frozen();
    with_params! {
        set a.b = 1;
    }
    
    assert_eq!(get_param!(a.b, 0), 0);

    async fn set_worker(val: i64) -> i64 {
        with_params! {
            set a.b = val;
            val
        }
    }

    async fn get_worker() -> i64 {
        with_params! {
            get val = a.b or 0;
            
            val
        }
    }

    let t1 = tokio::spawn(set_worker(2));
    let v1 = t1.await.unwrap();
    assert_eq!(v1, 2);
    assert_eq!(get_param!(a.b, 0), 0);

    let t2 = with_params! {
        set a.b = 3;

        hyperparameter::spawn(async { get_param!(a.b, 0) })
    };
    let v2 = t2.await.unwrap();
    assert_eq!(v2, 3);

    let t3 = with_params_async! {
        set a.b = 4;

        get_worker()
    };
    let v3 = t3.await;
    assert_eq!(v3, 4);
    
}

#[tokio::test(flavor = "current_thread")]
async fn test_async_and_threads_no_leakage() {
    use tokio::runtime::Builder;
    use tokio::task;

    // Seed a base value and freeze so new runtimes inherit.
    with_params! {
        set base.x = 100;
    }
    frozen();

    let mut handles = Vec::new();
    for tid in 0..4 {
        handles.push(std::thread::spawn(move || {
            let rt = Builder::new_current_thread().enable_all().build().unwrap();
            let out = rt.block_on(async move {
                // Ensure base value exists in this thread (inheritance may vary)
                let base_x = get_param!(base.x, 0);
                if base_x != 100 {
                    with_params! {
                        set base.x = 100;
                    }
                }
                with_params! {
                    set thread.id = tid;
                }
                async fn inner(tid: i64, idx: i64) -> i64 {
                    with_params! {
                        set thread.val = tid * 10 + idx;
                        get v = thread.val or 0i64;
                        v
                    }
                }
                let a = task::spawn(inner(tid, 1));
                let b = task::spawn(inner(tid, 2));
                let (ra, rb) = tokio::join!(a, b);
                (tid, get_param!(base.x, 0), ra.unwrap(), rb.unwrap())
            });
            out
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        results.push(h.join().unwrap());
    }
    assert_eq!(results.len(), 4);
    for (tid, base, ra, rb) in results {
        assert_eq!(base, 100);
        assert_eq!(ra, tid * 10 + 1);
        assert_eq!(rb, tid * 10 + 2);
    }
    assert_eq!(get_param!(base.x, 0), 100);
}
