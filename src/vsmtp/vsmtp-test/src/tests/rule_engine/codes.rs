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
use crate::{run_test, tests::protocol::auth::unsafe_auth_config};
use base64::{engine::general_purpose::STANDARD, Engine};

run_test! {
    fn deny_message_1,
    input = [
        "HELO someone\r\n",
        "MAIL FROM:<a@satan.org>\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "501 4.7.1 satan is blacklisted on this server\r\n",
    ],
    hierarchy_builder = |builder| Ok(builder.add_root_filter_rules(include_str!("custom_codes_deny.vsl"))?.build()),
}

run_test! {
    fn deny_message_2,
    input = [
        "HELO someone\r\n",
        "MAIL FROM:<a@evil.com>\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "501 4.7.1 evil is blacklisted on this server\r\n",
    ],
    hierarchy_builder = |builder| Ok(builder.add_root_filter_rules(include_str!("custom_codes_deny.vsl"))?.build()),
}

run_test! {
    fn deny_message_3,
    input = [
        "HELO someone\r\n",
        "MAIL FROM:<a@unpleasant.eu>\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "501 4.7.1 unpleasant is blacklisted on this server\r\n",
    ],
    hierarchy_builder = |builder| Ok(builder.add_root_filter_rules(include_str!("custom_codes_deny.vsl"))?.build()),
}

run_test! {
    fn accept_message,
    input = [
        "HELO client.com\r\n",
        &format!("AUTH PLAIN {}\r\n", STANDARD.encode(format!("\0{}\0{}", "admin", "password"))),
        "MAIL FROM:<admin@company.com>\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250 Ok\r\n",
        "235 2.7.0 Authentication succeeded\r\n",
        "250 welcome aboard chief\r\n",
        "221 Service closing transmission channel\r\n"
    ],
    config = unsafe_auth_config(),
    hierarchy_builder = |builder| Ok(builder.add_root_filter_rules(include_str!("custom_codes_accept.vsl"))?.build()),
}
