use hyperparameter::*;
use std::thread::{self, JoinHandle};

#[test]
fn test_with_params() {
    with_params! {
        set a.int = 1;
        set a.float = 2.0;
        set a.bool = true;
        set a.str = "string".to_string();

        with_params! {
            get a_int = a.int or 0;

            assert_eq!(1, a_int);
        };
    }
}

#[test]
fn test_with_params_multi_threads() {
    with_params! {
        set a.int = 1;
        set a.float = 2.0;
        set a.bool = true;
        set a.str = "string".to_string();

        frozen_global_params();

        let mut workers: Vec<JoinHandle<()>> = Vec::new();
        for _ in 0..10 {
            let t = thread::spawn(||{
                for i in 0..100000 {
                    with_params! {
                        get x = a.int or 0;
                        assert!(x == 1 );

                        with_params!{
                            set a.int = i%10;
                        };
                    };
                }
            });
            workers.push(t);
        }

        for t in workers {
            let _ = t.join();
        }
    }
}
