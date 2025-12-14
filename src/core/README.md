
# High-Performance Rust Configuration System

Hyperparameter is a high-performance configuration system designed for Rust and Python, supporting the following features:

1. **High Performance**: Provides fast parameter access, allowing users to freely read and write parameters in the code without worrying about performance issues.
2. **Scope Management**: Manages the definition and use of parameters through scopes, ensuring the isolation and safety of parameter values.
3. **Command Line Integration**: Automatically displays all parameters and their help information in the application's command line.

## Minimal Example

Here is a simple example demonstrating how to use Hyperparameter to build a command-line program:

```rust
use clap::Parser;
use hyperparameter::*;

#[derive(Parser)]
#[command(after_long_help=generate_params_help())]
struct CommandLineArgs {
    /// Specifies parameters in the format `-D key=value` on the command line
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = CommandLineArgs::parse();
    with_params! {
        params ParamScope::from(&args.define); // Receives all parameters from the command line

        // Retrieves the parameter `example.param1`, using a default value of `1` if not specified.
        println!("param1={}", get_param!(example.param1, 1));
        // Retrieves the parameter `example.param2`, displaying help information when `<app> --help` is executed.
        println!("param2={}", get_param!(example.param2, false, "help for example.param2"));
    }
}
```
When executing `clap_mini --help`, a section `Hyperparameters` appears at the end of the help information, explaining the names of hyperparameters and their help information:

```
Usage: clap_mini [OPTIONS]

Options:
  -D, --define <DEFINE>
          Specifies hyperparameters in the format `-D key=value` via the command line

  -h, --help
          Print help (see a summary with '-h')

Hyperparameters:
  example.param2
        help for example.param2
```
Following the prompt, you can specify the parameter value using `-D example.param2=<value>`:

```shell
$ clap_mini # Default values
param1=1
param2=false

$ clap_mini -D example.param2=true
param1=1
param2=true
```

## Using Configuration Files

Hyperparameter also supports the use of configuration files. The following example shows how to integrate configuration files, command-line parameters, and user-defined configurations:

```rust
use std::path::Path;

use clap::Parser;
use config::{self, File};
use hyperparameter::*;

#[derive(Parser)]
#[command(after_long_help=generate_params_help())]
struct CommandLineArgs {
    /// Specifies parameters in the format `-D key=value` on the command line
    #[arg(short = 'D', long)]
    define: Vec<String>,

    /// Specifies the configuration file path in the format `-C <path>` on the command line
    #[arg(short = 'C', long, default_value = "examples/rust/cfg.toml")]
    config: String,
}

fn main() {
    let args = CommandLineArgs::parse();
    let config_path = Path::new(&args.config);
    let config = config::Config::builder()
        .add_source(File::from(config_path))
        .build().unwrap();

    println!("param1={} // No scope", get_param!(example.param1, "default".to_string()));

    with_params! { // Configuration file parameter scope
        params config.param_scope();

        println!("param1={} // cfg file scope", get_param!(example.param1, "default".to_string()));
        with_params! { // Command-line arguments scope
            params ParamScope::from(&args.define);

            println!("param1={} // cmdline args scope", get_param!(example.param1, "default".to_string(), "Example param1"));
            with_params! { // User-defined scope
                @set example.param1= "scoped".to_string();

                println!("param1={} // user-defined scope", get_param!(example.param1, "default".to_string()));
            }
        }
    }
}
```
Directly executing the command `clap_layered` yields the following output:

```
param1=default     // No scope            # Outside any specific scope
param1=from config // cfg file scope      # Entered configuration file scope, parameter value affected by the config file
param1=from config // cmdline args scope  # Entered command-line scope, command-line overrides config file
param1=scoped      // user-defined scope  # Entered user-defined scope, custom value overrides command-line
```
As can be seen:
1. Nested scopes override layer by layer, with parameters in an inner scope overriding those in an outer scope.
2. The command-line scope did not specify the parameter, thus inheriting the value from the outer scope.

If the command line specifies the value of `example.param1`, the following input is obtained:

```shell
$ clap_layered -D example.param1="from cmdline"
param1=default  // No scope
param1=from config      // cfg file scope
param1=from cmdline     // cmdline args scope
param1=scoped   // user-defined scope
```