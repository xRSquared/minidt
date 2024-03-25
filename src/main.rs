use anyhow::Result;

mod cli;
mod config;
mod constants;
mod styles;
mod template;
mod utils;

fn main() -> Result<()> {
    let args = cli::parse_args();
    // NOTE: we can move config loading here, but would need to create a new function in config.rs
    match args.command {
        | cli::Command::Init(init_args) => config::init_project(init_args),
        // | cli::Command::Compile(compile_args) => compile_sql(compile_args),
        | cli::Command::Compile(compile_args) => {
            let input_path = compile_args.file;
            let output_path = compile_args.output;
            template::compile_template(&input_path, output_path.as_deref())
        },
    }
}
