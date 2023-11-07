use std::io::Error;

use config;
use hyperparameter::*;

fn main() -> Result<(), Error> {
    let cfg = config::Config::builder()
        .add_source(config::File::from_str(
            r#"{
            "a": 1,
            "b": "2",
            "foo": {
              "a": 11,
              "b": "22"
            }
        }"#,
            config::FileFormat::Json,
        ))
        .build()
        .unwrap();
    with_params! {
        params cfg.param_scope();

        with_params! {
            get a = a or 0i64;
            get b = b or String::from("0");
            get foo_a = foo.a or 0i64;

            println!("a={}", a);
            println!("b={}", b);
            println!("foo.a={}", foo_a);
        }
    }
    Ok(())
}
