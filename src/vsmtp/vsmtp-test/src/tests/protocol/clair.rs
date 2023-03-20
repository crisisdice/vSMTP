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
use crate::config;
use crate::recv_handler_wrapper::OnMessageCompletedHook;
use crate::run_test;
use vsmtp_common::{addr, ContextFinished};
use vsmtp_mail_parser::{BodyType, Mail, MailHeaders, MailMimeParser, MessageBody};

// see https://datatracker.ietf.org/doc/html/rfc5321#section-4.3.2

run_test! {
    fn test_receiver_1,
    input = [
        "HELO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n",
        "RCPT TO:<aa@bb>\r\n",
        "DATA\r\n",
        ".\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    mail_handler = |ctx: ContextFinished, _: MessageBody| {
        assert_eq!(ctx.helo.client_name.to_string(), "foobar");
        assert_eq!(ctx.mail_from.reverse_path, Some(addr!("john@doe")));
        assert!(ctx.rcpt_to.delivery
            .values()
            .flatten()
            .map(|(addr, _)| addr)
            .cloned()
            .eq([
                addr!("aa@bb")
            ])
        );
    }
}

run_test! {
    fn test_receiver_2,
    input = ["foo\r\n", "QUIT\r\n"],
    expected = [
        "220 testserver.com Service ready\r\n",
        "500 Syntax error command unrecognized\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn test_receiver_3,
    input = ["MAIL FROM:<john@doe>\r\n", "QUIT\r\n"],
    expected = [
        "220 testserver.com Service ready\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn test_receiver_4,
    input = ["RCPT TO:<john@doe>\r\n", "QUIT\r\n"],
    expected = [
        "220 testserver.com Service ready\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn test_receiver_5,
    input = ["HELO foo\r\n", "RCPT TO:<bar@foo>\r\n", "QUIT\r\n"],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn test_receiver_6,
    input = ["HELO foobar\r\n", "QUIT\r\n"],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ]
}

run_test! {
    fn test_receiver_10,
    input = ["HELP\r\n", "QUIT\r\n"],
    expected = [
        "220 testserver.com Service ready\r\n",
        "214 joining us https://viridit.com/support\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn test_receiver_11,
    input = [
        "HELO postmaster\r\n",
        "MAIL FROM: <doe@foo>\r\n",
        "RCPT TO: <doe@foo>\r\n",
        "DATA\r\n",
        ".\r\n",
        "DATA\r\n",
        "MAIL FROM:<b@b>\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "503 Bad sequence of commands\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn test_receiver_11_bis,
    input = [
        "HELO postmaster\r\n",
        "MAIL FROM: <doe@foo>\r\n",
        "RCPT TO: <doe@foo>\r\n",
        "DATA\r\n",
        ".\r\n",
        "DATA\r\n",
        "RCPT TO:<b@b>\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "503 Bad sequence of commands\r\n",
        "503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n"
    ]
}

run_test! {
    fn max_rcpt_reached,
    input = [
        "EHLO client.com\r\n",
        "MAIL FROM:<foo@bar.com>\r\n",
        "RCPT TO:<foo+1@bar.com>\r\n",
        "RCPT TO:<foo+2@bar.com>\r\n",
        "RCPT TO:<foo+3@bar.com>\r\n",
        "RCPT TO:<foo+4@bar.com>\r\n",
        "RCPT TO:<foo+5@bar.com>\r\n",
        "RCPT TO:<foo+6@bar.com>\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250 SMTPUTF8\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "452 Requested action not taken: too many recipients\r\n",
        "221 Service closing transmission channel\r\n"
    ],
    config = {
        let mut config = config::local_test();
        config.server.smtp.rcpt_count_max = 5;
        config
    }
}

run_test! {
    fn test_receiver_13,
    input = [
        "HELO foobar\r\n",
        "MAIL FROM:<john1@doe>\r\n",
        "RCPT TO:<aa1@bb>\r\n",
        "DATA\r\n",
        concat!(
            "from: john1 doe <john1@doe>\r\n",
            "date: tue, 30 nov 2021 20:54:27 +0100\r\n",
            "\r\n",
            "mail 1\r\n",
            ".\r\n",
        ),
        "MAIL FROM:<john2@doe>\r\n",
        "RCPT TO:<aa2@bb>\r\n",
        "DATA\r\n",
        concat!(
            "from: john2 doe <john2@doe>\r\n",
            "date: tue, 30 nov 2021 20:54:27 +0100\r\n",
            "\r\n",
            "mail 2\r\n",
            ".\r\n",
        ),
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    mail_handler = {
        #[derive(Clone)]
        struct T { count: std::sync::Arc<std::sync::atomic::AtomicU32> }

        impl OnMessageCompletedHook for T {
            fn on_message_completed(self, ctx: ContextFinished, mut msg: MessageBody) {
                let count = self.count.load(std::sync::atomic::Ordering::Relaxed);
                assert_eq!(ctx.helo.client_name.to_string(), "foobar");
                assert_eq!(
                    ctx.mail_from.reverse_path,
                    Some(addr!(&format!("john{count}@doe")))
                );
                assert!(ctx.rcpt_to.delivery
                    .values()
                    .flatten()
                    .map(|(addr, _)| addr)
                    .cloned()
                    .eq([
                        addr!(&format!("aa{count}@bb"))
                    ])
                );

                pretty_assertions::assert_eq!(
                    *msg.parsed::<MailMimeParser>().unwrap(),
                    Mail {
                        headers: MailHeaders(
                            [
                                ("from", format!("john{count} doe <john{count}@doe>")),
                                ("date", "tue, 30 nov 2021 20:54:27 +0100".to_string()),
                            ]
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v))
                            .collect::<Vec<_>>()
                        ),
                         body: BodyType::Regular(vec![format!("mail {count}")])
                    }
                );

                self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }

        T { count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(1)) }
    },
}

run_test! {
    fn test_receiver_14,
    input = [
        "HELO foobar1\r\n",
        "MAIL FROM:<john1@doe>\r\n",
        "RCPT TO:<aa1@bb>\r\n",
        "DATA\r\n",
        concat!(
            "from: john1 doe <john1@doe>\r\n",
            "date: tue, 30 nov 2021 20:54:27 +0100\r\n",
            "\r\n",
            "mail 1\r\n",
            ".\r\n",
        ),
        "HELO foobar2\r\n",
        "MAIL FROM:<john2@doe>\r\n",
        "RCPT TO:<aa2@bb>\r\n",
        "DATA\r\n",
        concat!(
            "from: john2 doe <john2@doe>\r\n",
            "date: tue, 30 nov 2021 20:54:27 +0100\r\n",
            "\r\n",
            "mail 2\r\n",
            ".\r\n",
        ),
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    mail_handler = {
        #[derive(Clone)]
        struct T { count: std::sync::Arc<std::sync::atomic::AtomicU32> }

        impl OnMessageCompletedHook for T {
            fn on_message_completed(self, ctx: ContextFinished, mut msg: MessageBody) {
                let count = self.count.load(std::sync::atomic::Ordering::Relaxed);
                assert_eq!(ctx.helo.client_name.to_string(), format!("foobar{count}"));
                assert_eq!(
                    ctx.mail_from.reverse_path,
                    Some(addr!(&format!("john{count}@doe")))
                );

                assert!(ctx.rcpt_to.delivery
                    .values()
                    .flatten()
                    .map(|(addr, _)| addr)
                    .cloned()
                    .eq([
                        addr!(&format!("aa{count}@bb"))
                    ])
                );

                pretty_assertions::assert_eq!(
                    *msg.parsed::<MailMimeParser>().unwrap(),
                    Mail {
                        headers: MailHeaders(
                            [
                                (
                                    "from",
                                    format!("john{count} doe <john{count}@doe>")
                                ),
                                ("date", "tue, 30 nov 2021 20:54:27 +0100".to_string()),
                            ]
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v))
                            .collect::<Vec<_>>()
                        ),
                        body: BodyType::Regular(vec![format!("mail {count}")])
                    }
                );

                self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
        T { count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(1)) }
    },
}

#[tokio::test]
async fn test_receiver_9() {
    let mut config = config::local_test();
    config.server.smtp.error.delay = std::time::Duration::from_millis(100);
    config.server.smtp.error.soft_count = 5;
    config.server.smtp.error.hard_count = 10;

    let config = std::sync::Arc::new(config);

    let before_test = std::time::Instant::now();
    run_test! {
        input = [
            "RCPT TO:<bar@foo>\r\n",
            "MAIL FROM: <foo@bar>\r\n",
            "EHLO\r\n",
            "NOOP\r\n",
            "azeai\r\n",
            "STARTTLS\r\n",
            "MAIL FROM:<john@doe>\r\n",
            "EHLO\r\n",
            "EHLO\r\n",
            "HELP\r\n",
            "aieari\r\n",
            "not a valid smtp command\r\n",
        ],
        expected = [
            "220 testserver.com Service ready\r\n",
            "503 Bad sequence of commands\r\n",
            "503 Bad sequence of commands\r\n",
            "500 Syntax error command unrecognized\r\n",
            "250 Ok\r\n",
            "500 Syntax error command unrecognized\r\n",
            "454 TLS not available due to temporary reason\r\n",
            "503 Bad sequence of commands\r\n",
            "500 Syntax error command unrecognized\r\n",
            "500 Syntax error command unrecognized\r\n",
            "214 joining us https://viridit.com/support\r\n",
            "500 Syntax error command unrecognized\r\n",
            "451-Syntax error command unrecognized\r\n",
            "451 Too many errors from the client\r\n"
        ],
        config_arc = config.clone(),
    };

    assert!(
        before_test.elapsed().as_millis()
            >= config.server.smtp.error.delay.as_millis()
                * u128::try_from(
                    config.server.smtp.error.hard_count - config.server.smtp.error.soft_count
                )
                .unwrap()
    );
}
