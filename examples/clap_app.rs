use clap::Parser;
use hyperparameter::*;

fn foo() {
    with_params! {
        get param1 = example.param1 or 1; // read param `example.param1` or use default value `1`
        get param2 = example.param2 or String::from("2"); // read param `example.param2` or use default value `2`
        get param3 = example.param3 or false; // read param `example.param3` or use default value `false`

        println!("param1={}", param1);
        println!("param2={}", param2);
        println!("param3={}", param3);
        println!("param4={}", get_param!(param4, 1, "help message for param4")); // leave a help msg when reading params
        println!("param4={}", get_param!(param4, 1, "another help for param4")); // leave another help msg
    }
}

#[derive(Parser, Debug)]
#[command(after_long_help=generate_params_help())]
struct DeriveArgs {
    /// define hyperparameters
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = DeriveArgs::parse();
    with_params! {
        params ParamScope::from(&args.define);

        foo()
    }
}
