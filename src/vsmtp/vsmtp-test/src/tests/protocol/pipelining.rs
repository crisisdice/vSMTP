/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms &of the GNU General Public License as published by the Free Software
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
use crate::run_test;

run_test! {
    fn accepting_pipelining,
    input = [
        "EHLO foobar\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",

        "221 Service closing transmission channel\r\n",
    ],
}

run_test! {
    fn basic_pipelining_scenario,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<galvin@tis.com>\r\n\
        DATA\r\n",
        &("X".repeat(10) + "\r\n.\r\n"),
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        250 Ok\r\n\
        354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ],
}

run_test! {
    fn sneaky_unrecognized_command, // TODO: check what to do in this case
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<fitz@trusted>\r\n\
        NOTACOMMAND and its args\
        DATA\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        250 Ok\r\n\
        503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n",

    ],
}

run_test! {
    fn no_rcpt,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        DATA\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        503 Bad sequence of commands\r\n",
        "221 Service closing transmission channel\r\n",

    ],
}

run_test! {
    fn wrong_rcpt,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<galvin@>\r\n\
        DATA\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        554 permanent problems with the remote server\r\n",
        "221 Service closing transmission channel\r\n",
    ],
}

run_test! {
    fn multiple_rcpt,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<henry@trusted.com>\r\n\
        RCPT TO:<galvin@trusted.com>\r\n\
        DATA\r\n",
        &("X".repeat(10) + "\r\n.\r\n"),
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        250 Ok\r\n\
        554 permanent problems with the remote server\r\n\
        354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ]
}

run_test! {
    fn multiple_rcpt_with_some_wrong,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<henry@trusted.com>\r\n\
        RCPT TO:<galvin@malicious.com>\r\n\
        DATA\r\n",
        &("X".repeat(10) + "\r\n.\r\n"),
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        250 Ok\r\n\
        554 permanent problems with the remote server\r\n\
        354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ]
}

run_test! {
    fn reset_after_data,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<henry@trusted.com>\r\n\
        RCPT TO:<galvin@malicious.com>\r\n\
        DATA\r\n",
        &("X".repeat(10) + "\r\n.\r\nRESET\r\n"),
        "MAIL FROM:<john@doe>\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        250 Ok\r\n\
        554 permanent problems with the remote server\r\n\
        354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ]
}

// FIXME: this test
run_test! {
    fn error_after_data,
    input = [
        "EHLO foobar\r\n",
        "MAIL FROM:<john@doe>\r\n\
        RCPT TO:<henry@trusted.com>\r\n\
        RCPT TO:<galvin@malicious.com>\r\n\
        DATA\r\n",
        &("X".repeat(10) + "\r\n.\r\nnotacommand\r\n"),
        "MAIL FROM:<john@doe>\r\n",
        "QUIT\r\n",
    ],
    expected = [
        "220 testserver.com Service ready\r\n",
        "250-testserver.com\r\n",
        "250-STARTTLS\r\n",
        "250-8BITMIME\r\n",
        "250-SMTPUTF8\r\n",
        "250 PIPELINING\r\n",
        "250 Ok\r\n\
        250 Ok\r\n\
        554 permanent problems with the remote server\r\n\
        354 Start mail input; end with <CRLF>.<CRLF>\r\n",
        "250 Ok\r\n",
        "250 Ok\r\n",
        "221 Service closing transmission channel\r\n",
    ]
}

/* Command statuses MUST be coordinated with responses by counting each
separate response and correlating that count with the number of
commands known to have been issued. Multiline responses MUST be
supported. Matching on the basis of either the error code value or
associated text is expressly forbidden. */

// MAIL FROM:<john@doe>
// RCPT TO:<galvin@n>
// DATA
