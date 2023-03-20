/*
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
*/

use crate::run_test;
use vsmtp_common::addr;
use vsmtp_common::{ContextFinished, TransactionType};
use vsmtp_mail_parser::MessageBody;

// TODO: add examples with outgoing & internal transaction types.
run_test! {
    fn test_aliases,
    input = [
        "HELO foo\r\n",
        "MAIL FROM: <someone@example.com>\r\n",
        "RCPT TO: <jenny@mydomain.com>\r\n",
        "RCPT TO: <joe@mydomain.com>\r\n",
        "RCPT TO: <john@gmail.com>\r\n",
        "RCPT TO: <oliver@mydomain.com>\r\n",
        "DATA\r\n",
        concat!(
            "From: <someone@example.com>\r\n",
            "To: jenny@mydomain.com, joe@mydomain.com, john@gmail.com, oliver@mydomain.com\r\n",
            "Subject: test\r\n",
            "\r\n",
            "test\r\n",
            ".\r\n",
        ),
        "QUIT\r\n"
    ],
    expected = [
        "220 mydomain.com Service ready\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        // fallback called because gmail.com isn't handled.
        "554 5.7.1 Relay access denied\r\n",
    ],
    config = vsmtp_config::Config::from_vsl_file(std::path::PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        "../../../examples/alias/vsmtp.vsl"
    ]))
    .unwrap(),
    mail_handler = |ctx: ContextFinished, _: MessageBody| {
        let fp = ctx.rcpt_to.delivery;

        assert_eq!(fp.len(), 2);
        assert_eq!(ctx.rcpt_to.transaction_type, TransactionType::Incoming(Some("mydomain.com".parse().unwrap())));
        assert!(fp.values().flatten().map(|(addr, _)| addr).cloned().eq([
            addr!("oliver@mydomain.com"),
            addr!("john.doe@mydomain.com")
        ]));


        // FIXME: logical error: adding a recipient with `add_rcpt_envelop` should take
        //        the `transaction_type` field into account, which it does not do for now.
        // assert_eq!(fp[1].transaction_type, TransactionType::Incoming(Some("mydomain.com".to_owned())));
    },
}
