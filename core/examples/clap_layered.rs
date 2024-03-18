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

fn main() {
    let args = CommandLineArgs::parse();
    let config_path = Path::new(&args.config);
    let config = config::Config::builder()
        .add_source(File::from(config_path))
        .build()
        .unwrap();

    // No scope
    println!(
        "param1={}\t// No scope",
        get_param!(example.param1, "default".to_string())
    );

    with_params! { // Scope with configuration file parameters
        params config.param_scope();

        println!("param1={}\t// cfg file scope", get_param!(example.param1, "default".to_string()));
        with_params! { // Scope with command-line arguments
            params ParamScope::from(&args.define);

            println!("param1={}\t// cmdline args scope", get_param!(example.param1, "default".to_string(), "Example param1"));
            with_params! { // User-defined scope
                set example.param1= "scoped".to_string();

                println!("param1={}\t// user-defined scope", get_param!(example.param1, "default".to_string()));
            }
        }
    }
}
