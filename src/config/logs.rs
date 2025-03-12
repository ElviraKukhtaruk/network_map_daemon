use log4rs::{
    append::{console::ConsoleAppender, rolling_file::{
        policy::compound::{roll::delete::DeleteRoller, trigger::size::SizeTrigger, CompoundPolicy},
        RollingFileAppender,
    }},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use log::LevelFilter;
use clap::Parser;
use super::parse_cli;


pub fn configure_logs() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli::Cli::parse();

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}")))
        .build();

    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}")))
        .build(
            &cli.logs_path,
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(10 * 1024 * 1024)),
                Box::new(DeleteRoller::new()),
            )),
        )?;

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(console_appender)))
        .appender(Appender::builder().build("file_logger", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file_logger")
                .build(LevelFilter::Info),
        )?;

    log4rs::init_config(config)?;
    Ok(())
}
