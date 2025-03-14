use log4rs::{
    append::{console::ConsoleAppender, rolling_file::{
        policy::compound::{roll::delete::DeleteRoller, trigger::size::SizeTrigger, CompoundPolicy},
        RollingFileAppender,
    }},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use log::LevelFilter;


pub fn configure_logs(logs_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {

    // Build the console appender with a specific log pattern
        let console_appender = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}")))
            .build();

        // Start building the config with the console appender
        let mut config_builder = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(console_appender)));
        let mut root_builder = Root::builder().appender("stdout");

        // If a log file path is provided, add the file appender
        if let Some(path) = logs_path {
            let file_appender = RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}")))
                .build(
                    path,
                    Box::new(CompoundPolicy::new(
                        Box::new(SizeTrigger::new(10 * 1024 * 1024)), // 10 MB size trigger
                        Box::new(DeleteRoller::new()),                // Delete old logs
                    )),
                )?;
            config_builder = config_builder.appender(Appender::builder().build("file_logger", Box::new(file_appender)));
            root_builder = root_builder.appender("file_logger");
        }
        let conf = config_builder.build(root_builder.build(LevelFilter::Info))?;
        log4rs::init_config(conf)?;
        Ok(())
}
