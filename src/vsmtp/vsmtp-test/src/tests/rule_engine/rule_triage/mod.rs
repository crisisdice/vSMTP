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

//! This suit of test aims to check if the rule engine dispatch rules correctly following
//! The email's transaction type.

use crate::run_test;
use vsmtp_common::{ContextFinished, TransactionType};
use vsmtp_mail_parser::MessageBody;

run_test! {
    fn test_incoming,
    input = [
        "HELO foo\r\n",
        "MAIL FROM: <john.doe@other.com>\r\n",
        "RCPT TO: <green@example.com>\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "100 incoming main\r\n",
        "100 incoming example.com\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    config = vsmtp_config::Config::from_vsl_file(std::path::PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        "src/tests/rule_engine/rule_triage/config/vsmtp.vsl"
    ])).unwrap(),
}

run_test! {
    fn test_outgoing_internal,
    input = [
        "HELO foo\r\n",
        "MAIL FROM: <john.doe@example.com>\r\n",
        "RCPT TO: <green@example.com>\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "100 sender outgoing example.com\r\n",
        "100 internal example.com\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    config = vsmtp_config::Config::from_vsl_file(std::path::PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        "src/tests/rule_engine/rule_triage/config/vsmtp.vsl"
    ])).unwrap()
}

const INTERNAL_EMAIL: &str = r#"X-CUSTOM: An internal email
X-INTERNAL: green@example.com
From: john.doe@example.com
To: green@example.com, bar@other.com
Date: 0

Hi !
"#;

const OUTGOING_EMAIL: &str = r#"From: john.doe@example.com
To: green@example.com, bar@other.com
Date: 0
X-OUTGOING: bar@other.com

Hi !
"#;

run_test! {
    fn test_split_email_outgoing_internal,
    input = [
        "HELO foo\r\n",
        "MAIL FROM: <john.doe@example.com>\r\n",
        "RCPT TO: <green@example.com>\r\n",
        "RCPT TO: <bar@other.com>\r\n",
        "DATA\r\n",
        concat!(
            "From: john.doe@example.com\r\n",
            "To: green@example.com, bar@other.com\r\n",
            "Date: 0\r\n",
            "\r\n",
            "Hi !\r\n",
            ".\r\n",
        ),
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "100 sender outgoing example.com\r\n",
        "100 internal example.com\r\n",
        "100 rcpt outgoing example.com\r\n",
        "354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250-Ok\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    config = vsmtp_config::Config::from_vsl_file(std::path::PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        "src/tests/rule_engine/rule_triage/config/vsmtp.vsl"
    ])).unwrap(),
    mail_handler = |ctx: ContextFinished, body: MessageBody| {
        match ctx.rcpt_to.transaction_type {
            TransactionType::Internal => {
                assert_eq!(body.get_header("X-OUTGOING"), None);
                assert_eq!(body.get_header("X-INTERNAL"), Some("green@example.com".to_owned()));
                assert_eq!(body.get_header("X-CUSTOM"), Some("An internal email".to_owned()));
                assert_eq!(body.inner().to_string(), INTERNAL_EMAIL.replace('\n', "\r\n"));
            },
            TransactionType::Outgoing { domain } => {
                assert_eq!(domain, "example.com".parse().unwrap());
                assert_eq!(body.get_header("X-OUTGOING"), Some("bar@other.com".to_owned()));
                assert_eq!(body.get_header("X-INTERNAL"), None);
                assert_eq!(body.get_header("X-CUSTOM"), None);
                assert_eq!(body.inner().to_string().as_str(), OUTGOING_EMAIL.replace('\n', "\r\n"));
            },
            TransactionType::Incoming(_) => panic!("should be outgoing / internal"),
        }
    },
}

run_test! {
    fn test_fallback_to_root_domain,
    input = [
        "HELO foo\r\n",
        "MAIL FROM: <john.doe@any.com>\r\n",
        // Should fallback to 'example.com' incoming rules.
        "RCPT TO: <green@unknown.example.com>\r\n",
        "RCPT TO: <green@mta.example.com>\r\n",
        "QUIT\r\n"
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "100 incoming main\r\n",
        "100 incoming example.com\r\n",
        "100 incoming mta.example.com\r\n",
        "221 Service closing transmission channel\r\n",
    ],
    config = vsmtp_config::Config::from_vsl_file(std::path::PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        "src/tests/rule_engine/rule_triage/config/vsmtp.vsl"
    ])).unwrap(),
}
