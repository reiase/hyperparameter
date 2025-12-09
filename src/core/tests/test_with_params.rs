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
        }
    }
}

#[test]
fn test_with_params_multi_threads() {
    with_params! {
        set a.int = 1;
        set a.float = 2.0;
        set a.bool = true;
        set a.str = "string".to_string();

        frozen();

        let mut workers: Vec<JoinHandle<()>> = Vec::new();
        for _ in 0..10 {
            let t = thread::spawn(|| {
                for i in 0..100000 {
                    with_params! {
                        get x = a.int or 0;
                        assert!(x == 1);

                        with_params! {
                            set a.int = i % 10;
                        }
                    }
                }
            });
            workers.push(t);
        }

        for t in workers {
            let _ = t.join();
        }
    }
}

#[test]
fn test_with_params_nested() {
    with_params! {
        set a.b = 1;
        
        let outer: i64 = get_param!(a.b, 0);
        assert_eq!(1, outer);
        
        with_params! {
            set a.b = 2;
            
            let inner: i64 = get_param!(a.b, 0);
            assert_eq!(2, inner);
        }
        
        let restored: i64 = get_param!(a.b, 0);
        assert_eq!(1, restored);
    }
}

#[test]
fn test_with_params_expression() {
    let result = with_params! {
        set demo.val = 1;
        
        let x: i64 = get_param!(demo.val, 0);
        x + 1
    };
    assert_eq!(2, result);
}
