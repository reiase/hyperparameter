use std::path::Path;

use clap::Parser;
use config::{self, File};
use hyperparameter::*;

#[derive(Parser)]
#[command(after_long_help=generate_params_help())]
struct CommandLineArgs {
    /// Specifies hyperparameters in the format `-D key=value` via the command line.
    #[arg(short = 'D', long)]
    define: Vec<String>,

    /// Specifies the configuration file path.
    #[arg(short = 'C', long, default_value = "examples/cfg.toml")]
    config: String,
}

fn foo(desc: &str) {
    with_params! {
        // Example param1 - this is shown in help
        @get param1 = example.param1 or "default".to_string();

        println!("param1={} // {}", param1, desc);
    }
}

fn main() {
    let args = CommandLineArgs::parse();
    let config_path = Path::new(&args.config);
    let config = config::Config::builder()
        .add_source(File::from(config_path))
        .build()
        .unwrap();

    foo("Outside any specific scope");

    with_params! { // Scope with configuration file parameters
        params config.param_scope();

        foo("Within configuration file scope");

        with_params! { // Scope with command-line arguments
            params ParamScope::from(&args.define);

            foo("Within command-line arguments scope");

            with_params! { // User-defined scope
                @set example.param1 = "scoped".to_string();

                foo("Within user-defined scope");
            }
        }
    }
}
