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
use vsmtp_common::{status::Status, Reply};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;

enum BlockListKinds {
    Spamhaus,
    Spamrats,
    Default
}

fn blocklist_values(bl: &String) -> BlockListKinds {
    match bl.as_str() {
        "spamhaus" => BlockListKinds::Spamhaus,
        "spamrats" => BlockListKinds::Spamrats,
        _ => BlockListKinds::Default
    }
}

fn blocklist_urls(bl: BlockListKinds) -> Option<String> {
    match bl {
        BlockListKinds::Spamhaus => Some(String::from("zen.spamhaus.org")),
        BlockListKinds::Spamrats => Some(String::from("all.spamrats.com")),
        BlockListKinds::Default => None
    }
}

#[derive(Debug, serde::Deserialize)]
struct DnsblParameters {
    pub bl: Vec<String>
}

pub struct Dnsbl {
    pub bl: Vec<String>
}

impl Dnsbl {
    pub fn check(&self, domain: String) -> Result<bool, Box<rhai::EvalAltResult>> {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
        let mut result = false;
        for element in &self.bl {
            let bl_kind = blocklist_values(element);
            let response = resolver.lookup_ip(domain.clone() + "." + blocklist_urls(bl_kind).unwrap().as_str());
            match response {
                Ok(res) => {
                    dbg!(res);
                    result = true;
                    break;
                },
                Err(err) => {
                    dbg!(err);
                    result = false;
                }
            }
        }
        Ok(result)
    }

    pub fn lookup(&self, domain: String) -> Result<Status, Box<rhai::EvalAltResult>> {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
        for element in &self.bl {
            let bl_kind = blocklist_values(element);
            let response = resolver.lookup_ip(domain.clone() + "." + blocklist_urls(bl_kind).unwrap().as_str());
            match response {
                Ok(res) => {
                    dbg!(res);
                    return Ok(Status::Deny(
                        "XXX spam detected\r\n"
                            .parse::<Reply>()
                            .unwrap(),
                    ));
                },
                Err(err) => {
                    dbg!(err);
                }
            }
        }
        Ok(Status::Next)
    }
}

#[rhai::plugin::export_module]
pub mod vsmtp_plugin_dnsbl {
    pub type Bl = rhai::Shared<Dnsbl>;

    #[rhai_fn(global, return_raw)]
    pub fn build(parameters: rhai::Map) -> Result<Bl, Box<rhai::EvalAltResult>> {
        let parameters = rhai::serde::from_dynamic::<DnsblParameters>(&parameters.into())?;
        let mut bl_list = Vec::<String>::new();

        println!("Building DNSBL with bl:");
        for element in parameters.bl {
            bl_list.push(dbg!(element));
        }
        Ok(rhai::Shared::new(Dnsbl {
            bl: bl_list
        }))
    }

    #[rhai_fn(global, return_raw, pure)]
    pub fn check(
        con: &mut Bl,
        domain: rhai::Dynamic
    ) -> Result<bool, Box<rhai::EvalAltResult>> {
        con.check(domain.to_string())
    }

    #[rhai_fn(global, return_raw, pure)]
    pub fn lookup(
        con: &mut Bl,
        domain: rhai::Dynamic
    ) -> Result<Status, Box<rhai::EvalAltResult>> {
        con.lookup(domain.to_string())
    }
}
