use clap::Parser;
use log::error;
use routee_compass::app::cli::cli_args::CliArgs;
use routee_compass::app::cli::run;
use routee_compass::app::compass::CompassAppBuilder;

fn main() {
    env_logger::init();

    let args = CliArgs::parse();
    let builder = CompassAppBuilder::default();
    match run::command_line_runner(&args, Some(builder), None) {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e)
        }
    }
}
