use std::collections::HashSet;

/**
 * vSMTP mail transfer agent
 * Copyright (C) 2021 viridIT SAS
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
use super::io_service::{IoService, ReadError};
use crate::config::log::{RECEIVER, RULES};
use crate::config::server_config::{ServerConfig, TlsSecurityLevel};
use crate::model::envelop::Envelop;
use crate::model::mail::{ConnectionData, MailContext};
use crate::resolver::DataEndResolver;
use crate::rules::address::Address;
use crate::rules::rule_engine::{RuleEngine, Status};
use crate::smtp::code::SMTPReplyCode;
use crate::smtp::event::Event;

/// Abstracted memory of the last client message
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum StateSMTP {
    Connect,
    Helo,
    NegotiationTLS,
    MailFrom,
    RcptTo,
    Data,
    Stop,
}

impl std::fmt::Display for StateSMTP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StateSMTP::Connect => "Connect",
            StateSMTP::Helo => "Helo",
            StateSMTP::NegotiationTLS => "NegotiationTLS",
            StateSMTP::MailFrom => "MailFrom",
            StateSMTP::RcptTo => "RcptTo",
            StateSMTP::Data => "Data",
            StateSMTP::Stop => "Stop",
        })
    }
}

pub struct StateSMTPFromStrError;

impl std::fmt::Display for StateSMTPFromStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("SourceFromStrError")
    }
}

impl std::str::FromStr for StateSMTP {
    type Err = StateSMTPFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Connect" => Ok(StateSMTP::Connect),
            "Helo" => Ok(StateSMTP::Helo),
            "MailFrom" => Ok(StateSMTP::MailFrom),
            "RcptTo" => Ok(StateSMTP::RcptTo),
            "Data" => Ok(StateSMTP::Data),
            _ => Err(StateSMTPFromStrError),
        }
    }
}

const MAIL_CAPACITY: usize = 10_000_000; // 10MB

// TODO: move that cleanly in config
const TIMEOUT_DEFAULT: u64 = 10_000; // 10s

pub struct MailReceiver<'a, R>
where
    R: DataEndResolver,
{
    /// config
    server_config: std::sync::Arc<ServerConfig>,
    tls_config: Option<std::sync::Arc<rustls::ServerConfig>>,
    smtp_timeouts: std::collections::HashMap<StateSMTP, std::time::Duration>,

    /// rule engine executing the server's rhai configuration.
    rule_engine: RuleEngine<'a>,

    /// Current connection data
    state: StateSMTP,
    mail: MailContext,
    error_count: u64,
    is_secured: bool,

    /// cached state // TODO: move that cleanly in config
    next_line_timeout: std::time::Duration,

    _phantom: std::marker::PhantomData<R>,
}

impl<R> MailReceiver<'_, R>
where
    R: DataEndResolver,
{
    pub fn new(
        peer_addr: std::net::SocketAddr,
        tls_config: Option<std::sync::Arc<rustls::ServerConfig>>,
        server_config: std::sync::Arc<ServerConfig>,
    ) -> Self {
        // TODO: move that cleanly in config
        let smtp_timeouts = server_config
            .smtp
            .timeout_client
            .iter()
            .filter_map(|(k, v)| match humantime::parse_duration(v) {
                Ok(v) => Some((*k, v)),
                Err(e) => {
                    log::error!(
                        target: RECEIVER,
                        "error \"{}\" parsing timeout for key={}, ignored",
                        e,
                        k
                    );
                    None
                }
            })
            .collect::<std::collections::HashMap<_, _>>();

        Self {
            state: StateSMTP::Connect,
            rule_engine: RuleEngine::new(server_config.as_ref()),
            mail: MailContext {
                connection: ConnectionData {
                    peer_addr,
                    timestamp: std::time::SystemTime::now(),
                },
                envelop: Envelop::default(),
                body: String::with_capacity(MAIL_CAPACITY),
                timestamp: None,
            },
            tls_config,
            is_secured: false,
            next_line_timeout: *smtp_timeouts
                .get(&StateSMTP::Connect)
                .unwrap_or(&std::time::Duration::from_millis(TIMEOUT_DEFAULT)),
            smtp_timeouts,
            server_config,
            error_count: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn set_helo(&mut self, helo: String) {
        self.mail.envelop = Envelop {
            helo,
            mail_from: Address::default(),
            rcpt: HashSet::default(),
        };
        self.rule_engine.reset();

        self.rule_engine
            .add_data("helo", self.mail.envelop.helo.clone());
    }

    fn set_mail_from(&mut self, mail_from: String) {
        if let Ok(mail_from) = Address::new(&mail_from) {
            self.mail.envelop.mail_from = mail_from;
            self.mail.timestamp = Some(std::time::SystemTime::now());
            self.mail.envelop.rcpt.clear();
            self.rule_engine.reset();

            self.rule_engine
                .add_data("mail", self.mail.envelop.mail_from.clone());
            self.rule_engine
                .add_data("mail_timestamp", self.mail.timestamp);
        }
    }

    // FIXME: too many clone
    fn set_rcpt_to(&mut self, rcpt_to: String) {
        if let Ok(rcpt_to) = Address::new(&rcpt_to) {
            self.rule_engine.add_data("rcpt", rcpt_to.clone());

            match self.rule_engine.get_data::<HashSet<Address>>("rcpts") {
                Some(mut rcpts) => {
                    rcpts.insert(rcpt_to);
                    self.mail.envelop.rcpt = rcpts.clone();
                    self.rule_engine.add_data("rcpts", rcpts.clone());
                }
                None => unreachable!("rcpts is injected by the default scope"),
            };
        } else {
            log::error!(target: RECEIVER, "rcpt's email address is invalid.");
        }
    }

    async fn process_event(&mut self, event: Event) -> (Option<StateSMTP>, Option<SMTPReplyCode>) {
        match (&self.state, event) {
            (_, Event::NoopCmd) => (None, Some(SMTPReplyCode::Code250)),

            (_, Event::HelpCmd(_)) => (None, Some(SMTPReplyCode::Code214)),

            (_, Event::RsetCmd) => {
                self.mail.body = String::with_capacity(MAIL_CAPACITY);
                self.mail.envelop.rcpt.clear();
                self.mail.envelop.mail_from = Address::default();
                self.rule_engine.reset();

                (Some(StateSMTP::Helo), Some(SMTPReplyCode::Code250))
            }

            (_, Event::ExpnCmd(_) | Event::VrfyCmd(_) | Event::PrivCmd) => {
                (None, Some(SMTPReplyCode::Code502unimplemented))
            }

            (_, Event::QuitCmd) => (Some(StateSMTP::Stop), Some(SMTPReplyCode::Code221)),

            (_, Event::HeloCmd(helo)) => {
                self.set_helo(helo);
                log::trace!(
                    target: RECEIVER,
                    "[p:{}] envelop=\"{:?}\"",
                    self.mail.connection.peer_addr.port(),
                    self.mail.envelop,
                );

                let status = self.rule_engine.run_when("helo");
                self.process_rules_status(
                    status,
                    Some(StateSMTP::Helo),
                    Some(SMTPReplyCode::Code250),
                )
            }

            (_, Event::EhloCmd(_)) if self.server_config.smtp.disable_ehlo => {
                (None, Some(SMTPReplyCode::Code502unimplemented))
            }

            (_, Event::EhloCmd(helo)) => {
                self.set_helo(helo);
                log::trace!(
                    target: RECEIVER,
                    "[p:{}] envelop=\"{:?}\"",
                    self.mail.connection.peer_addr.port(),
                    self.mail.envelop,
                );

                let status = self.rule_engine.run_when("helo");
                self.process_rules_status(
                    status,
                    Some(StateSMTP::Helo),
                    Some(if self.is_secured {
                        SMTPReplyCode::Code250SecuredEsmtp
                    } else {
                        SMTPReplyCode::Code250PlainEsmtp
                    }),
                )
            }

            (StateSMTP::Helo, Event::StartTls) if self.tls_config.is_some() => (
                Some(StateSMTP::NegotiationTLS),
                Some(SMTPReplyCode::Code220),
            ),

            (StateSMTP::Helo, Event::StartTls) if self.tls_config.is_none() => {
                (None, Some(SMTPReplyCode::Code454))
            }

            (StateSMTP::Helo, Event::MailCmd(_))
                if self.server_config.tls.security_level == TlsSecurityLevel::Encrypt
                    && !self.is_secured =>
            {
                (None, Some(SMTPReplyCode::Code530))
            }

            (StateSMTP::Helo, Event::MailCmd(mail_from)) => {
                self.mail.body = String::with_capacity(MAIL_CAPACITY);
                self.set_mail_from(mail_from);

                log::trace!(
                    target: RECEIVER,
                    "[p:{}] envelop=\"{:?}\"",
                    self.mail.connection.peer_addr.port(),
                    self.mail.envelop,
                );

                let status = self.rule_engine.run_when("mail");
                self.process_rules_status(
                    status,
                    Some(StateSMTP::MailFrom),
                    Some(SMTPReplyCode::Code250),
                )
            }

            (StateSMTP::MailFrom | StateSMTP::RcptTo, Event::RcptCmd(rcpt_to)) => {
                self.set_rcpt_to(rcpt_to);

                log::trace!(
                    target: RECEIVER,
                    "[p:{}] envelop=\"{:?}\"",
                    self.mail.connection.peer_addr.port(),
                    self.mail.envelop,
                );

                let status = self.rule_engine.run_when("rcpt");
                let result = self.process_rules_status(
                    status,
                    Some(StateSMTP::RcptTo),
                    Some(SMTPReplyCode::Code250),
                );

                match self.server_config.smtp.rcpt_count_max {
                    Some(rcpt_count_max) if rcpt_count_max < self.mail.envelop.rcpt.len() => (
                        Some(StateSMTP::RcptTo),
                        Some(SMTPReplyCode::Code452TooManyRecipients),
                    ),
                    _ => result,
                }
            }

            (StateSMTP::RcptTo, Event::DataCmd) => {
                (Some(StateSMTP::Data), Some(SMTPReplyCode::Code354))
            }

            (StateSMTP::Data, Event::DataLine(line)) => {
                self.mail.body.push_str(&line);
                self.mail.body.push('\n');
                (None, None)
            }

            (StateSMTP::Data, Event::DataEnd) => {
                self.rule_engine.add_data("data", self.mail.body.clone());

                let status = self.rule_engine.run_when("preq");

                let result = match status {
                    Status::Block => return (Some(StateSMTP::Stop), Some(SMTPReplyCode::Code554)),
                    _ => self.process_rules_status(
                        status,
                        Some(StateSMTP::MailFrom),
                        Some(SMTPReplyCode::Code250),
                    ),
                };

                // checking if the rule engine haven't ran successfully.
                match result {
                    (Some(StateSMTP::MailFrom), Some(SMTPReplyCode::Code250)) => {}
                    _ => return result,
                };

                // executing all registered extensive operations.
                if let Err(error) = self.rule_engine.execute_operation_queue(
                    &self.mail,
                    &format!(
                        "{}_{:?}",
                        self.mail
                            .timestamp
                            .unwrap()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        std::thread::current().id()
                    ),
                ) {
                    log::error!(
                        target: RULES,
                        "failed to empty the operation queue: '{}'",
                        error
                    );
                }

                // getting the server's envelop, that could have mutated in the
                // rule engine.
                if let Some(envelop) = self.rule_engine.get_scoped_envelop() {
                    self.mail.envelop = envelop;

                    // TODO: resolver should not be responsible for mutating the SMTP state
                    // should return a code and handle if code.is_error()
                    // NOTE: clear envelop and mail context ?
                    match R::on_data_end(&self.server_config, &self.mail).await {
                        Ok(code) => (Some(StateSMTP::Helo), Some(code)),
                        Err(error) => todo!("{}", error),
                    }
                } else {
                    // NOTE: which code is returned when the server failed ?
                    (Some(StateSMTP::MailFrom), Some(SMTPReplyCode::Code554))
                }
            }

            _ => (None, Some(SMTPReplyCode::Code503)),
        }
    }

    /// checks the result of the rule engine and returns the appropriate state and code.
    fn process_rules_status(
        &mut self,
        status: Status,
        desired_state: Option<StateSMTP>,
        desired_code: Option<SMTPReplyCode>,
    ) -> (Option<StateSMTP>, Option<SMTPReplyCode>) {
        match status {
            Status::Deny => (Some(StateSMTP::Stop), Some(SMTPReplyCode::Code554)),
            _ => (desired_state, desired_code),
        }
    }

    /// handle a clear text received with plain_stream or tls_stream
    async fn handle_plain_text(&mut self, client_message: String) -> Option<SMTPReplyCode> {
        log::trace!(
            target: RECEIVER,
            "[p:{}] buffer=\"{}\"",
            self.mail.connection.peer_addr.port(),
            client_message
        );

        let command_or_code = if self.state == StateSMTP::Data {
            Event::parse_data
        } else {
            Event::parse_cmd
        }(&client_message);

        log::trace!(
            target: RECEIVER,
            "[p:{}] parsed=\"{:?}\"",
            self.mail.connection.peer_addr.port(),
            command_or_code
        );

        let (new_state, reply) = match command_or_code {
            Ok(event) => self.process_event(event).await,
            Err(error) => (None, Some(error)),
        };

        if let Some(new_state) = new_state {
            log::info!(
                target: RECEIVER,
                "[p:{}] ================ STATE: /{:?}/ => /{:?}/",
                self.mail.connection.peer_addr.port(),
                self.state,
                new_state
            );
            self.state = new_state;
            self.next_line_timeout = *self
                .smtp_timeouts
                .get(&self.state)
                .unwrap_or(&std::time::Duration::from_millis(TIMEOUT_DEFAULT));
        }

        reply
    }

    async fn read_and_handle<S>(&mut self, io: &mut IoService<'_, S>) -> Result<(), std::io::Error>
    where
        S: std::io::Write + std::io::Read,
    {
        match tokio::time::timeout(self.next_line_timeout, io.get_next_line_async()).await {
            Ok(Ok(client_message)) => {
                if let Some(response) = self.handle_plain_text(client_message).await {
                    log::info!(
                        target: RECEIVER,
                        "[p:{}] send=\"{:?}\"",
                        self.mail.connection.peer_addr.port(),
                        response
                    );

                    if response.is_error() {
                        self.error_count += 1;

                        let hard_error = self.server_config.smtp.error.hard_count;
                        let soft_error = self.server_config.smtp.error.soft_count;

                        if hard_error != -1 && self.error_count >= hard_error as u64 {
                            let mut response_begin = self
                                .server_config
                                .smtp
                                .get_code()
                                .get(&response)
                                .to_string();
                            response_begin.replace_range(3..4, "-");
                            response_begin.push_str(
                                self.server_config
                                    .smtp
                                    .get_code()
                                    .get(&SMTPReplyCode::Code451TooManyError),
                            );
                            std::io::Write::write_all(io, response_begin.as_bytes())?;

                            return Err(std::io::Error::new(
                                std::io::ErrorKind::ConnectionAborted,
                                "too many errors",
                            ));
                        }

                        std::io::Write::write_all(
                            io,
                            self.server_config.smtp.get_code().get(&response).as_bytes(),
                        )?;

                        if soft_error != -1 && self.error_count >= soft_error as u64 {
                            std::thread::sleep(self.server_config.smtp.error.delay);
                        }
                    } else {
                        std::io::Write::write_all(
                            io,
                            self.server_config.smtp.get_code().get(&response).as_bytes(),
                        )?;
                    }
                }
                Ok(())
            }
            Ok(Err(ReadError::Blocking)) => Ok(()),
            Ok(Err(ReadError::Eof)) => {
                log::info!(
                    target: RECEIVER,
                    "[p:{}] (secured:{}) eof",
                    self.mail.connection.peer_addr.port(),
                    self.is_secured
                );
                self.state = StateSMTP::Stop;
                Ok(())
            }
            Ok(Err(ReadError::Other(e))) => {
                log::error!(
                    target: RECEIVER,
                    "[p:{}] (secured:{}) error {}",
                    self.mail.connection.peer_addr.port(),
                    self.is_secured,
                    e
                );
                self.state = StateSMTP::Stop;
                Err(e)
            }
            Err(e) => {
                std::io::Write::write_all(
                    io,
                    self.server_config
                        .smtp
                        .get_code()
                        .get(&SMTPReplyCode::Code451Timeout)
                        .as_bytes(),
                )?;
                Err(std::io::Error::new(std::io::ErrorKind::TimedOut, e))
            }
        }
    }

    fn complete_tls_handshake<S>(
        io: &mut IoService<rustls::Stream<rustls::ServerConnection, S>>,
        timeout: &std::time::Duration,
    ) -> Result<(), std::io::Error>
    where
        S: std::io::Read + std::io::Write,
    {
        let begin_handshake = std::time::Instant::now();

        while io.inner.conn.is_handshaking() {
            if begin_handshake.elapsed() > *timeout {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "too long",
                ));
            }
            match std::io::Write::flush(&mut io.inner) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    async fn receive_secured<S>(&mut self, mut plain_stream: S) -> Result<S, std::io::Error>
    where
        S: std::io::Read + std::io::Write,
    {
        let mut tls_connection = rustls::ServerConnection::new(
            self.tls_config
                .as_ref()
                .expect("Failed to get tsl_config")
                .clone(),
        )
        .expect("Failed to open tsl connection");

        let mut tls_stream: rustls::Stream<rustls::ServerConnection, S> =
            rustls::Stream::new(&mut tls_connection, &mut plain_stream);

        let mut io = IoService::new(&mut tls_stream);

        Self::complete_tls_handshake(&mut io, &self.server_config.tls.handshake_timeout)?;

        // TODO: rfc:
        // The decision of whether or not to believe the authenticity of the
        // other party in a TLS negotiation is a local matter.  However, some
        // general rules for the decisions are:
        //
        // -  A SMTP client would probably only want to authenticate an SMTP
        //    server whose server certificate has a domain name that is the
        //    domain name that the client thought it was connecting to.
        // -  A publicly-referenced  SMTP server would probably want to accept
        //    any verifiable certificate from an SMTP client, and would possibly
        //    want to put distinguishing information about the certificate in
        //    the Received header of messages that were relayed or submitted
        //    from the client.

        log::debug!(
            target: RECEIVER,
            "[p:{}] protocol_version={:#?}\n alpn_protocol={:#?}\n negotiated_cipher_suite={:#?}\n peer_certificates={:#?}\n sni_hostname={:#?}",
            self.mail.connection.peer_addr.port(),
            io.inner.conn.protocol_version(),
            io.inner.conn.alpn_protocol(),
            io.inner.conn.negotiated_cipher_suite(),
            io.inner.conn.peer_certificates(),
            io.inner.conn.sni_hostname(),
        );

        log::warn!(
            target: RECEIVER,
            "[p:{}] ================ STATE: /{:?}/ => /{:?}/",
            self.mail.connection.peer_addr.port(),
            self.state,
            StateSMTP::Connect
        );

        self.mail.envelop = Envelop::default();
        self.mail.body = String::with_capacity(MAIL_CAPACITY);

        self.state = StateSMTP::Connect;
        self.is_secured = true;
        self.next_line_timeout = *self
            .smtp_timeouts
            .get(&self.state)
            .unwrap_or(&std::time::Duration::from_millis(TIMEOUT_DEFAULT));

        while self.state != StateSMTP::Stop {
            self.read_and_handle(&mut io).await?;
        }

        Ok(plain_stream)
    }

    pub async fn receive_plain<S>(&mut self, mut plain_stream: S) -> Result<S, std::io::Error>
    where
        S: std::io::Write + std::io::Read,
    {
        let mut io = IoService::new(&mut plain_stream);

        match std::io::Write::write_all(
            &mut io,
            self.server_config
                .smtp
                .get_code()
                .get(&SMTPReplyCode::Code220)
                .as_bytes(),
        ) {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    target: RECEIVER,
                    "Error on sending response (receiving); error = {:?}",
                    e
                );
                return Err(e);
            }
        }

        self.rule_engine
            .add_data("connect", self.mail.connection.peer_addr.ip());
        self.rule_engine
            .add_data("port", self.mail.connection.peer_addr.port());
        self.rule_engine
            .add_data("connection_timestamp", self.mail.connection.timestamp);

        if let Status::Deny = self.rule_engine.run_when("connect") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "connection at '{}' has been denied when connecting.",
                    self.mail.connection.peer_addr
                ),
            ));
        };

        while self.state != StateSMTP::Stop {
            if self.state == StateSMTP::NegotiationTLS {
                return self.receive_secured(plain_stream).await;
            }
            self.read_and_handle(&mut io).await?;
        }
        Ok(plain_stream)
    }
}
