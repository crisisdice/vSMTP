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
use crate::tests::protocol::auth::unsafe_auth_config;
use vsmtp_common::addr;
use vsmtp_common::ContextFinished;
use vsmtp_mail_parser::MessageBody;

run_test! {
    fn getters,
    input = [
        "EHLO foo\r\n",
        "AUTH ANONYMOUS dG9rZW5fYWJjZGVm\r\n",
        "MAIL FROM:<replace@example.com>\r\n",
        "RCPT TO:<test@example.com>\r\n",
        "DATA\r\n",
        ".\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-AUTH PLAIN LOGIN CRAM-MD5 ANONYMOUS\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250 SMTPUTF8\r\n",
        "235 2.7.0 Authentication succeeded\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250-Ok\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n"
    ],
    config = unsafe_auth_config(),
    mail_handler = |ctx: ContextFinished, _: MessageBody| {
        assert_eq!(
            Some(addr!("john.doe@example.com")),
            ctx.mail_from.reverse_path
        );

        if matches!(&ctx.rcpt_to.transaction_type,
            vsmtp_common::TransactionType::Outgoing { domain }
                if *domain == "example.com".parse::<vsmtp_common::Domain>().unwrap()) {
            assert!(ctx.rcpt_to.delivery
                .values()
                .flatten()
                .map(|(addr, _)| addr)
                .cloned()
                .eq([
                    addr!("add4@example.com"),
                    addr!("replace4@example.com"),
                ])
            );
        } else {
            assert!(ctx.rcpt_to.delivery
                .values()
                .flatten()
                .map(|(addr, _)| addr)
                .cloned()
                .eq([
                    addr!("test@example.com"),
                    addr!("add4@example.com"),
                    addr!("replace4@example.com"),
                ])
            );
        }
    },
    hierarchy_builder = |builder| {
        Ok(
            builder
                .add_root_filter_rules(include_str!("getters.vsl"))?
                    .add_domain_rules("example.com".parse().unwrap())
                        .with_incoming(include_str!("getters.vsl"))?
                        .with_outgoing(include_str!("getters.vsl"))?
                        .with_internal(include_str!("getters.vsl"))?
                    .build()
                .build()
        )
    },
}
