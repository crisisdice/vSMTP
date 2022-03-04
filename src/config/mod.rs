/**
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 *  This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
**/
mod config_builder;
/// The default values of the configuration
pub mod default;
mod serializer;
/// The rust representation of the configuration
pub mod server_config;
/// The external services used in .vsl format
pub mod service;

#[cfg(test)]
mod tests;

pub(crate) mod log_channel {
    pub(crate) const RECEIVER: &str = "receiver";
    pub(crate) const RESOLVER: &str = "resolver";
    pub(crate) const SRULES: &str = "rules";
    pub(crate) const URULES: &str = "user_rules";
    pub(crate) const DELIVER: &str = "deliver";
}

/// helper to initialize the log4rs config from our ServerConfig
pub fn get_logger_config(config: &server_config::ServerConfig) -> anyhow::Result<log4rs::Config> {
    use log4rs::*;

    if config.log.file == config.rules.logs.file {
        anyhow::bail!(
            "rules and application logs cannot both be written in {:?}!",
            config.log.file
        );
    }

    let console = append::console::ConsoleAppender::builder()
        .encoder(Box::new(encode::pattern::PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l:<5} {I})} ((line:{L:<3})) $ {m}{n}",
        )))
        .build();

    let app = append::file::FileAppender::builder()
        .encoder(Box::new(encode::pattern::PatternEncoder::new(
            "{d} - {m}{n}",
        )))
        .build(config.log.file.clone())?;

    let user = append::file::FileAppender::builder()
        .encoder(Box::new(encode::pattern::PatternEncoder::new(
            config
                .rules
                .logs
                .format
                .as_ref()
                .unwrap_or(&"{d} - {m}{n}".to_string()),
        )))
        .build(config.rules.logs.file.clone())?;

    Config::builder()
        .appender(config::Appender::builder().build("stdout", Box::new(console)))
        .appender(config::Appender::builder().build("app", Box::new(app)))
        .appender(config::Appender::builder().build("user", Box::new(user)))
        .loggers(
            config
                .log
                .level
                .iter()
                .map(|(name, level)| config::Logger::builder().build(name, *level)),
        )
        .logger(
            config::Logger::builder()
                .appender("user")
                .additive(false)
                .build(log_channel::URULES, config.rules.logs.level),
        )
        .build(
            config::Root::builder()
                .appender("stdout")
                .appender("app")
                .build(
                    *config
                        .log
                        .level
                        .get("default")
                        .unwrap_or(&log::LevelFilter::Warn),
                ),
        )
        .map_err(|e| {
            e.errors().iter().for_each(|e| log::error!("{}", e));
            anyhow::anyhow!(e)
        })
}