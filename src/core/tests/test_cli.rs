use hyperparameter::PARAMS;
use linkme::distributed_slice;

#[test]
fn test_cli() {
    #[distributed_slice(PARAMS)]
    static param1: (&str, &str) = (
        "key1", "val1"
    );

    assert!(PARAMS.len()==1);

    for kv in PARAMS {
        println!("{} => {}", kv.0, kv.1);
    }
}