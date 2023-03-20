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
use crate::run_test;
use vsmtp_common::addr;
use vsmtp_common::ContextFinished;
use vsmtp_mail_parser::BodyType;
use vsmtp_mail_parser::Mail;
use vsmtp_mail_parser::MailHeaders;
use vsmtp_mail_parser::MailMimeParser;
use vsmtp_mail_parser::MessageBody;

run_test! {
    fn reset_helo,
    input = [
        "HELO foo\r\n",
        "RSET\r\n",
        "MAIL FROM:<a@b>\r\n",
        "RCPT TO:<b@c>\r\n",
        "DATA\r\n",
        concat!(
            "from: a b <a@b>\r\n",
            "date: tue, 30 nov 2021 20:54:27 +0100\r\n",
            "\r\n",
            "mail content wow\r\n",
            ".\r\n",
        ),
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n"
    ],
    mail_handler = |ctx: ContextFinished, mut body: MessageBody| {
        assert_eq!(ctx.helo.client_name.to_string(), "foo");
        assert_eq!(ctx.mail_from.reverse_path, Some(addr!("a@b")));

        assert!(ctx.rcpt_to.delivery
            .values()
            .flatten()
            .map(|(addr, _)| addr)
            .cloned()
            .eq([
                addr!("b@c")
            ])
        );

        assert_eq!(
            *body.parsed::<MailMimeParser>().unwrap(),
            Mail {
                headers: MailHeaders(
                    [
                        ("from", "a b <a@b>"),
                        ("date", "tue, 30 nov 2021 20:54:27 +0100"),
                    ]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<_>>()
                ),
                body: BodyType::Regular(vec!["mail content wow".to_string()])
            }
        );
    },
}

run_test! {
    fn reset_mail_from_error,
    input = [
        "HELO foo\r\n",
        "MAIL FROM:<a@b>\r\n",
        "RSET\r\n",
        "RCPT TO:<b@c>\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ],
}

run_test! {
    fn reset_mail_ok,
    input = [
        "HELO foo\r\n",
        "MAIL FROM:<a@b>\r\n",
        "RSET\r\n",
        "HELO foo2\r\n",
        "RCPT TO:<b@c>\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ],
}

run_test! {
    fn reset_rcpt_to_ok,
    input = [
        "HELO foo\r\n",
        "MAIL FROM:<a@b>\r\n",
        "RSET\r\n",
        "HELO foo2\r\n",
        "MAIL FROM:<d@e>\r\n",
        "RCPT TO:<b@c>\r\n",
        "DATA\r\n",
        ".\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n"
    ],
    mail_handler = |ctx: ContextFinished, mut body: MessageBody| {
        assert_eq!(ctx.helo.client_name.to_string(), "foo2");
        assert_eq!(ctx.mail_from.reverse_path, Some(addr!("d@e")));

        assert!(ctx.rcpt_to.delivery
            .values()
            .flatten()
            .map(|(addr, _)| addr)
            .cloned()
            .eq([
                addr!("b@c")
            ])
        );

        assert_eq!(
            *body.parsed::<MailMimeParser>().unwrap(),
            Mail {
                headers: MailHeaders(vec![]),
                body: BodyType::Undefined
            }
        );
    },
}

run_test! {
    fn reset_rcpt_to_error,
    input = [
        "HELO foo\r\n",
        "MAIL FROM:<foo@foo>\r\n",
        "RCPT TO:<toto@bar>\r\n",
        "RSET\r\n",
        "RCPT TO:<toto2@bar>\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ],
}

run_test! {
    fn reset_rcpt_to_multiple_rcpt,
    input = [
        "HELO foo\r\n",
        "MAIL FROM:<foo@foo>\r\n",
        "RCPT TO:<toto@bar>\r\n",
        "RSET\r\n",
        "MAIL FROM:<foo2@foo>\r\n",
        "RCPT TO:<toto2@bar>\r\n",
        "RCPT TO:<toto3@bar>\r\n",
        "DATA\r\n",
        concat!(
            "from: foo2 foo <foo2@foo>\r\n",
            "date: tue, 30 nov 2021 20:54:27 +0100\r\n",
            ".\r\n",
        ),
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n"
    ],
    mail_handler = |ctx: ContextFinished, mut body: MessageBody| {
        assert_eq!(ctx.helo.client_name.to_string(), "foo");
        assert_eq!(ctx.mail_from.reverse_path, Some(addr!("foo2@foo")));
        assert!(ctx.rcpt_to.delivery
            .values()
            .flatten()
            .map(|(addr, _)| addr)
            .cloned()
            .eq([
                addr!("toto2@bar"),
                addr!("toto3@bar")
            ])
        );

        pretty_assertions::assert_eq!(
            *body.parsed::<MailMimeParser>().unwrap(),
            Mail {
                headers: MailHeaders(
                    [
                        ("from", "foo2 foo <foo2@foo>"),
                        ("date", "tue, 30 nov 2021 20:54:27 +0100"),
                    ]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<_>>()
                ),
                body: BodyType::Undefined
            }
        );
    },
}
