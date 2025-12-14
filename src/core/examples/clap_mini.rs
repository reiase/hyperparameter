use clap::Parser;
use hyperparameter::*;

#[derive(Parser)]
#[command(after_long_help=generate_params_help())] // Displays parameter helps when `<app> --help` is executed.
struct CommandLineArgs {
    /// Specifies hyperparameters in the format `-D key=value` via the command line.
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = CommandLineArgs::parse();
    with_params! {
        params ParamScope::from(&args.define);

        // Retrieves `example.param1` with a default value of `false` if not specified.
        let val: bool = get_param!(example.param1, false);
        println!("param1={}", val);
    }
}
