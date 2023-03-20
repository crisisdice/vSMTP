/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/
use crate::{
    receiver::ReceiverContext, smtp_sasl::CallbackWrap, AcceptArgs, AuthArgs, AuthError, EhloArgs,
    Error, HeloArgs, MailFromArgs, ParseArgsError, RcptToArgs, UnparsedArgs, Verb,
};
use tokio_rustls::rustls;
use vsmtp_common::ContextFinished;
// TODO: should we move these type in this crate
use vsmtp_common::{Reply, Stage};
use vsmtp_mail_parser::MessageBody;

// NOTE: could have 3 trait to make the implementation easier
// PreTransactionHandler + TransactionHandler + PostTransactionHandler

/// Trait to implement to handle the SMTP commands in pair with the [`Receiver`](crate::Receiver).
#[async_trait::async_trait]
pub trait ReceiverHandler {
    /// The [`Receiver`](crate::Receiver) does not store the context.
    /// This function is called after each command to get the context stage.
    fn get_stage(&self) -> Stage;

    /// Create an instance capable to handle the SASL handshake.
    fn generate_sasl_callback(&self) -> CallbackWrap;

    /// Called when the client connects to the server.
    async fn on_accept(&mut self, ctx: &mut ReceiverContext, args: AcceptArgs) -> Reply;

    /// Called after receiving a [`Verb::StartTls`] command.
    async fn on_starttls(&mut self, ctx: &mut ReceiverContext) -> Reply;

    /// Called after a successful TLS handshake.
    async fn on_post_tls_handshake(
        &mut self,
        sni: Option<String>,
        protocol_version: rustls::ProtocolVersion,
        cipher_suite: rustls::CipherSuite,
        peer_certificates: Option<Vec<rustls::Certificate>>,
        alpn_protocol: Option<Vec<u8>>,
    ) -> Reply;

    /// Called after receiving a [`Verb::Auth`] command.
    async fn on_auth(&mut self, ctx: &mut ReceiverContext, args: AuthArgs) -> Option<Reply>;

    /// Called after a successful SASL handshake.
    async fn on_post_auth(
        &mut self,
        ctx: &mut ReceiverContext,
        result: Result<(), AuthError>,
    ) -> Reply;

    /// Called after receiving a [`Verb::Helo`] command.
    async fn on_helo(&mut self, ctx: &mut ReceiverContext, args: HeloArgs) -> Reply;

    /// Called after receiving a [`Verb::Ehlo`] command.
    async fn on_ehlo(&mut self, ctx: &mut ReceiverContext, args: EhloArgs) -> Reply;

    /// Called after receiving a [`Verb::MailFrom`] command.
    async fn on_mail_from(&mut self, ctx: &mut ReceiverContext, args: MailFromArgs) -> Reply;

    /// Called after receiving a [`Verb::RcptTo`] command.
    async fn on_rcpt_to(&mut self, ctx: &mut ReceiverContext, args: RcptToArgs) -> Reply;

    /// Called after receiving a [`Verb::Data`] command.
    ///
    /// The stream is the body of the message, with dot-stuffing handled.
    /// The stream return `None` when the message is finished (`.<CRLF>`).
    async fn on_message(
        &mut self,
        ctx: &mut ReceiverContext,
        stream: impl tokio_stream::Stream<Item = Result<Vec<u8>, Error>> + Send + Unpin,
    ) -> (Reply, Option<Vec<(ContextFinished, MessageBody)>>);

    /// Called for each message produced by the [`ReceiverHandler::on_message()`] method.
    ///
    /// If this callback returns `Some`, the reply produced by [`ReceiverHandler::on_message()`] is discarded.
    async fn on_message_completed(
        &mut self,
        ctx: ContextFinished,
        msg: MessageBody,
    ) -> Option<Reply>;

    /// Called when the number of reply considered as error reached a threshold (hard).
    async fn on_hard_error(&mut self, ctx: &mut ReceiverContext, reply: Reply) -> Reply;

    /// Called when the number of reply considered as error reached a threshold (soft).
    async fn on_soft_error(&mut self, ctx: &mut ReceiverContext, reply: Reply) -> Reply;

    /// Called after receiving a [`Verb::Rset`] command.
    async fn on_rset(&mut self) -> Reply;

    /// Called after receiving a [`Verb::Data`] command.
    #[inline]
    async fn on_data(&mut self) -> Reply {
        #[allow(clippy::expect_used)]
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n"
            .parse()
            .expect("valid syntax")
    }

    /// Called after receiving a [`Verb::Quit`] command.
    #[inline]
    async fn on_quit(&mut self) -> Reply {
        #[allow(clippy::expect_used)]
        "221 Service closing transmission channel"
            .parse()
            .expect("valid syntax")
    }

    /// Called after receiving a [`Verb::Noop`] command.
    #[inline]
    async fn on_noop(&mut self) -> Reply {
        #[allow(clippy::expect_used)]
        "250 Ok\r\n".parse().expect("valid syntax")
    }

    /// Called after receiving a [`Verb::Help`] command.
    #[inline]
    async fn on_help(&mut self, _: UnparsedArgs) -> Reply {
        #[allow(clippy::expect_used)]
        "214 joining us https://viridit.com/support"
            .parse()
            .expect("valid syntax")
    }

    /// Called after receiving an unknown command (unrecognized or unimplemented).
    #[inline]
    async fn on_unknown(&mut self, buffer: Vec<u8>) -> Reply {
        let unimplemented_command = [b"VRFY".as_slice(), b"EXPN".as_slice(), b"TURN".as_slice()];

        #[allow(clippy::expect_used)]
        if unimplemented_command.iter().any(|c| {
            buffer.len() >= c.len()
                && buffer
                    .get(..c.len())
                    .expect("range checked before")
                    .eq_ignore_ascii_case(c)
        }) {
            "502 Command not implemented\r\n"
                .parse()
                .expect("valid syntax")
        } else {
            "500 Syntax error command unrecognized\r\n"
                .parse()
                .expect("valid syntax")
        }
    }

    /// Called when the stage of the transaction (obtained with [`get_stage`](Self::get_stage))
    /// and the command are not compatible.
    #[inline]
    async fn on_bad_sequence(&mut self, _: (Verb, Stage)) -> Reply {
        #[allow(clippy::expect_used)]
        "503 Bad sequence of commands\r\n"
            .parse()
            .expect("valid syntax")
    }

    /// Called when an argument of a command is invalid.
    #[inline]
    async fn on_args_error(&mut self, _: ParseArgsError) -> Reply {
        #[allow(clippy::expect_used)]
        "501 Syntax error in parameters or arguments\r\n"
            .parse()
            .expect("valid syntax")
    }
}
