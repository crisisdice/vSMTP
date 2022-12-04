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
//mod cipher_suite;
mod starttls;
mod tunneled;
mod tunneled_with_auth;

const TEST_SERVER_CERT: &str = "src/template/certs/certificate.crt";
const TEST_SERVER_KEY: &str = "src/template/certs/private_key.rsa.key";

use vsmtp_config::Config;

pub fn get_tls_config() -> Config {
    Config::builder()
        .with_version_str("<1.0.0")
        .unwrap()
        .without_path()
        .with_server_name("testserver.com")
        .with_user_group_and_default_system("root", "root")
        .unwrap()
        .with_ipv4_localhost()
        .with_default_logs_settings()
        .with_spool_dir_and_default_queues("./tmp/spool")
        .with_safe_tls_config(TEST_SERVER_CERT, TEST_SERVER_KEY)
        .unwrap()
        .with_default_smtp_options()
        .with_default_smtp_error_handler()
        .with_default_smtp_codes()
        .without_auth()
        .with_app_at_location("./tmp/app")
        .with_vsl("./src/template/ignore_vsl")
        .with_default_app_logs()
        .with_system_dns()
        .without_virtual_entries()
        .validate()
        .unwrap()
}