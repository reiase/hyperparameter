use clap::Parser;
use hyperparameter::*;

fn foo() {
    println!("in function foo");
    with_params! {
        get param1 = example.param1 or 1; // read param `example.param1` or use default value `1`
        get param2 = example.param2 or String::from("2"); // read param `example.param2` or use default value `2`
        get param3 = example.param3 or false; // read param `example.param3` or use default value `false`

        println!("param1={}", param1);
        println!("param2={}", param2);
        println!("param3={}", param3);
    }
}

fn bar() {
    println!("in function bar");
    foo()
}

#[derive(Parser, Debug)]
struct DeriveArgs {
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = DeriveArgs::parse();
    with_params! {
        params ParamScope::from(&args.define);

        bar()
    }
}
