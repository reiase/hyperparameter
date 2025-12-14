use hyperparameter::*;

#[test]
fn with_params_can_be_used_as_expression() {
    let result = with_params! {
        @set demo.val = 1;
        @get x = demo.val or 0;

        x + 1
    };
    assert_eq!(2, result);
}

#[test]
fn with_params_get_default() {
    let val: i64 = with_params! {
        // no set, should return default
        get_param!(missing.val, 42)
    };
    assert_eq!(42, val);
}

#[test]
fn with_params_mixed_set_get() {
    let result = with_params! {
        @set a.b = 10;
        @get val = a.b or 0;

        let doubled = val * 2;
        doubled
    };
    assert_eq!(20, result);
}
