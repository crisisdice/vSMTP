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

use rhai::plugin::*;

#[derive(Debug, serde::Deserialize)]
struct DnsblParameters {
    pub kind: String
}

pub struct Dnsbl {
    pub kind: String
}

impl Dnsbl {
    pub fn check(&self, domain: String) -> Result<bool, Box<rhai::EvalAltResult>> {
        Ok(true)
    }

    pub fn choose_db() {

    }

    pub fn translate_results() {

    }
}

#[rhai::plugin::export_module]
pub mod vsmtp_plugin_dnsbl {
    pub type Bl = rhai::Shared<Dnsbl>;

    #[rhai_fn(global, return_raw)]
    pub fn build(parameters: rhai::Map) -> Result<Bl, Box<rhai::EvalAltResult>> {
        let parameters = rhai::serde::from_dynamic::<DnsblParameters>(&parameters.into())?;

        println!("Building DNSBL with kind: {}", parameters.kind);
        Ok(rhai::Shared::new(Dnsbl {
            kind: parameters.kind
        }))
    }

    #[rhai_fn(global, return_raw, pure)]
    pub fn check(
        con: &mut Bl,
        domain: rhai::Dynamic
    ) -> Result<bool, Box<rhai::EvalAltResult>> {
        con.check(domain.to_string())
    }
}
