use hyperparameter::*;

#[test]
fn with_params_can_be_used_as_expression() {
    let result = with_params! {
        set demo.val = 1;
        get x = demo.val or 0;

        x + 1
    };
    assert_eq!(2, result);
}

#[test]
fn with_params_readonly_expression() {
    let doubled = with_params_readonly! {
        get x = missing.val or 3;

        x * 2
    };
    assert_eq!(6, doubled);
}
