# 高性能Rust配置系统

Hyperparameter 是一个为 Rust 和 Python 设计的高性能配置系统，它支持以下特性：

1. 高性能：提供快速的参数访问，允许用户在代码中自由读写参数，无需担心性能问题。
2. 作用域管理：通过作用域管理参数的定义和使用，确保参数值的隔离和安全。
3. 命令行集成：支持在应用的命令行中自动展示所有参数及其帮助信息。


## 最小示例

以下是一个简单示例，展示如何使用 Hyperparameter 构建一个命令行程序：
```rust
use clap::Parser;
use hyperparameter::*;

#[derive(Parser)]
#[command(after_long_help=generate_params_help())]
struct CommandLineArgs {
    /// 在命令行以`-D key=value`格式指定参数
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = CommandLineArgs::parse();
    with_params! {
        params ParamScope::from(&args.define); // 从命令行接收全部参数

        // 读取参数`example.param1`，若未指定则使用默认值`1`.
        println!("param1={}", get_param!(example.param1, 1));
        // 读取参数`example.param2`，在执行`<app> --help`时输出帮助信息.
        println!("param2={}", get_param!(example.param2, false, "help for example.param2"));
    }
}
```
当执行`clap_mini --help`时，在帮助信息结尾出现了`Hyperparameters`一节，说明了超参名称及其帮助信息:
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
根据提示，可以使用`-D example.param2=<value>`来指定参数取值：
```shell
$ clap_mini # 默认取值
param1=1
param2=false

$ clap_mini -D example.param2=true
param1=1
param2=true
```

## 结合配置文件使用

Hyperparameter 也支持与配置文件结合使用。以下示例展示了如何整合配置文件、命令行参数和用户自定义配置：

```rust
use std::path::Path;

use clap::Parser;
use config::{self, File};
use hyperparameter::*;

#[derive(Parser)]
#[command(after_long_help=generate_params_help())]
struct CommandLineArgs {
    /// 在命令行以`-D key=value`格式指定参数
    #[arg(short = 'D', long)]
    define: Vec<String>,

    /// 在命令行以`-C <path>`格式指定配置文件
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

    with_params! { // 配置文件参数作用域
        params config.param_scope();

        println!("param1={} // cfg file scope", get_param!(example.param1, "default".to_string()));
        with_params! { // 命令行参数作用域
            params ParamScope::from(&args.define);

            println!("param1={} // cmdline args scope", get_param!(example.param1, "default".to_string(), "Example param1"));
            with_params! { // 用户自定义作用域
                set example.param1= "scoped".to_string();

                println!("param1={} // user-defined scope", get_param!(example.param1, "default".to_string()));
            }
        }
    }
}

```
直接执行命令`clap_layered`后得到如下输出：
```
param1=default     // No scope            # 未进入任何scope
param1=from config // cfg file scope      # 进入配置文件scope，参数取值受配置文件影响
param1=from config // cmdline args scope  # 进入命令行scope，命令行覆盖配置文件
param1=scoped      // user-defined scope  # 进入自定义scope，自定义取值覆盖命令行
```
可以看到：
1. 嵌套的scope逐层覆盖，内层scope中参数覆盖外层scope；
2. 命令行scope未指定参数，因此继承了外层scope的取值

若使用命令行指定`example.param1`的取值，则得到如下输入：
```shell
$ clap_layered -D example.param1="from cmdline"
param1=default  // No scope
param1=from config      // cfg file scope
param1=from cmdline     // cmdline args scope
param1=scoped   // user-defined scope
```