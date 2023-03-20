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

use vsmtp_plugin_vsl::objects::Object;

use crate::api::{EngineResult, SharedObject};
use rhai::plugin::{
    mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

pub use utils::*;

/// Utility functions to interact with the system.
#[rhai::plugin::export_module]
mod utils {
    /// Get the root domain (the registrable part)
    ///
    /// # Examples
    ///
    /// `foo.bar.example.com` => `example.com`
    ///
    /// # rhai-autodocs:index:1
    #[rhai_fn()]
    #[must_use]
    pub fn get_root_domain(domain: &str) -> String {
        vsmtp_auth::get_root_domain(domain).map_or_else(|_| domain.to_string(), |root| root)
    }

    #[doc(hidden)]
    #[rhai_fn(name = "get_root_domain", pure, return_raw)]
    pub fn get_root_domain_obj(domain: &mut SharedObject) -> EngineResult<String> {
        match domain.as_ref() {
            Object::Fqdn(domain) => Ok(get_root_domain(domain)),
            _ => Err(format!("type `{}` is not a domain", domain.as_ref()).into()),
        }
    }

    /// Fetch an environment variable from the current process.
    ///
    /// # Args
    ///
    /// * `variable` - the variable to fetch.
    ///
    /// # Returns
    ///
    /// * `string` - the value of the fetched variable.
    /// * `()`     - when the variable is not set,  when the variable contains the sign character (=) or the NUL character,
    /// or that the variable does not contain valid Unicode.
    ///
    /// # Example
    ///
    /// ```
    /// # let states = vsmtp_test::vsl::run(
    /// # |builder| Ok(builder.add_root_filter_rules(r#"
    /// #{
    ///   connect: [
    ///     rule "get env variable" || {
    ///
    ///       // get the HOME environment variable.
    ///       let home = utils::env("HOME");
    ///
    /// #       if home == () {
    /// #           return state::deny(`500 home,${home}`);
    /// #       }
    ///
    ///       // "VSMTP=ENV" is malformed, this will return the unit type '()'.
    ///       let invalid = utils::env("VSMTP=ENV");
    ///
    /// #       if invalid != () {
    /// #           return state::deny(`500 invalid,${invalid}`);
    /// #       }
    ///
    /// #       state::accept(`250 test ok`)
    ///       // ...
    ///     }
    ///   ],
    /// }
    /// # "#)?.build()));
    /// # use vsmtp_common::{status::Status, CodeID, Reply, ReplyCode::Code};
    /// # assert_eq!(states[&vsmtp_rule_engine::ExecutionStage::Connect].2, Status::Accept(either::Right(
    /// #  "250 test ok".parse().unwrap(),
    /// # )));
    /// ```
    ///
    /// # rhai-autodocs:index:2
    #[rhai_fn(global, name = "env")]
    pub fn env_str(variable: &str) -> rhai::Dynamic {
        std::env::var(variable).map_or(rhai::Dynamic::UNIT, std::convert::Into::into)
    }

    #[doc(hidden)]
    #[rhai_fn(global, name = "env", pure)]
    pub fn env_obj(variable: &mut SharedObject) -> rhai::Dynamic {
        std::env::var(variable.to_string()).map_or(rhai::Dynamic::UNIT, std::convert::Into::into)
    }
}
