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

use crate::{
    api::{Context, EngineResult, Message, Server},
    get_global,
};
use rhai::plugin::{
    mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};
use vsmtp_auth::dkim as backend;
use vsmtp_common::Domain;
use vsmtp_mail_parser::MessageBody;

pub use dkim::*;

/// Parameters used by the [`sign`] function.
#[derive(Debug, serde::Deserialize)]
pub struct SignatureParams {
    sdid: Option<String>,
    selector: String,
    #[serde(deserialize_with = "deserialize_private_key")]
    private_key: std::sync::Arc<backend::PrivateKey>,
    headers_field: Option<Vec<String>>,
    #[serde(deserialize_with = "deserialize_canonicalization")]
    canonicalization: Option<backend::Canonicalization>,
}

fn deserialize_private_key<'de, D>(
    deserializer: D,
) -> Result<std::sync::Arc<backend::PrivateKey>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let private_key = <rhai::Dynamic as serde::Deserialize>::deserialize(deserializer)?;

    private_key
        .try_cast::<rhai::Shared<backend::PrivateKey>>()
        .ok_or_else(|| serde::de::Error::custom("failed to parse private key"))
}

fn deserialize_canonicalization<'de, D>(
    deserializer: D,
) -> Result<Option<backend::Canonicalization>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let canonicalization = <rhai::Dynamic as serde::Deserialize>::deserialize(deserializer)?;

    if canonicalization.is_unit() {
        return Ok(None);
    }

    canonicalization
        .into_string()
        .map_err(|t| {
            serde::de::Error::custom(format!(
                "dkim canonicalization parameter is not a string (got {t})"
            ))
        })?
        .parse()
        .map(Some)
        .map_err(|_| serde::de::Error::custom("failed to parse canonicalization"))
}

/// Generate and verify DKIM signatures.
/// Implementation of RFC 6376. (<https://www.rfc-editor.org/rfc/rfc6376.html>)
#[rhai::plugin::export_module]
mod dkim {
    /// Has the `ctx()` a DKIM signature verification result ?
    ///
    /// # rhai-autodocs:index:1
    #[rhai_fn(name = "has_result", return_raw)]
    pub fn has_result(ncc: NativeCallContext) -> EngineResult<bool> {
        super::Impl::has_dkim_result(&get_global!(ncc, ctx)?)
    }

    /// Return the DKIM signature verification result in the `ctx()` or
    /// an error if no result is found.
    ///
    /// # rhai-autodocs:index:2
    #[rhai_fn(name = "result", return_raw)]
    pub fn result(ncc: NativeCallContext) -> EngineResult<rhai::Map> {
        super::Impl::dkim_result(&get_global!(ncc, ctx)?)
    }

    /// Store the result produced by the DKIM signature verification in the `ctx()`.
    ///
    /// # Error
    /// * The `status` field is missing in the DKIM verification results.
    ///
    /// # rhai-autodocs:index:3
    #[rhai_fn(return_raw)]
    pub fn store(ncc: NativeCallContext, result: rhai::Map) -> EngineResult<()> {
        super::Impl::store(&get_global!(ncc, ctx)?, &result)
    }

    /// Get the list of DKIM private keys associated with this sdid
    ///
    /// # rhai-autodocs:index:4
    #[rhai_fn(return_raw)]
    pub fn get_private_keys(ncc: NativeCallContext, sdid: &str) -> EngineResult<rhai::Array> {
        let server = get_global!(ncc, srv)?;
        let r#virtual = server
            .config
            .server
            .r#virtual
            .get(
                &sdid
                    .parse::<Domain>()
                    .map_err::<Box<rhai::EvalAltResult>, _>(|e| e.to_string().into())?,
            )
            .and_then(|r#virtual| r#virtual.dkim.as_ref())
            .map(|dkim| {
                dkim.private_key
                    .iter()
                    .map(|key| rhai::Dynamic::from(key.inner.clone()))
                    .collect::<Vec<_>>()
            });

        Ok(r#virtual.unwrap_or_default())
    }

    /// return the `sdid` property of the [`backend::Signature`]
    ///
    /// # rhai-autodocs:index:5
    #[rhai_fn(global, get = "sdid", pure)]
    pub fn sdid(signature: &mut backend::Signature) -> String {
        signature.sdid.clone()
    }

    /// return the `auid` property of the [`backend::Signature`]
    ///
    /// # rhai-autodocs:index:6
    #[rhai_fn(global, get = "auid", pure)]
    pub fn auid(signature: &mut backend::Signature) -> String {
        signature.auid.clone()
    }

    /// Operate the hashing of the `message`'s headers and body, and compare the result with the
    /// `signature` and `key` data.
    ///
    /// # Examples
    ///
    /// ```
    /// // The message received.
    /// let msg = r#"
    /// Received: from github.com (hubbernetes-node-54a15d2.ash1-iad.github.net [10.56.202.84])
    /// 	by smtp.github.com (Postfix) with ESMTPA id 19FB45E0B6B
    /// 	for <mlala@negabit.com>; Wed, 26 Oct 2022 14:30:51 -0700 (PDT)
    /// DKIM-Signature: v=1; a=rsa-sha256; c=relaxed/relaxed; d=github.com;
    /// 	s=pf2014; t=1666819851;
    /// 	bh=7gTTczemS/Aahap1SpEnunm4pAPNuUIg7fUzwEx0QUA=;
    /// 	h=Date:From:To:Subject:From;
    /// 	b=eAufMk7uj4R+bO5Nr4DymffdGdbrJNza1+eykatgZED6tBBcMidkMiLSnP8FyVCS9
    /// 	 /GSlXME6/YffAXg4JEBr2lN3PuLIf94S86U3VckuoQQQe1LPtHlnGW5ZwJgi6DjrzT
    /// 	 klht/6Pn1w3a2jdNSDccWhk5qlSOQX9JKnE7UD58=
    /// Date: Wed, 26 Oct 2022 14:30:51 -0700
    /// From: Mathieu Lala <noreply@github.com>
    /// To: mlala@negabit.com
    /// Message-ID: <viridIT/vSMTP/push/refs/heads/test/rule-engine/000000-c6459a@github.com>
    /// Subject: [viridIT/vSMTP] c6459a: test: add test on message
    /// Mime-Version: 1.0
    /// Content-Type: text/plain;
    ///  charset=UTF-8
    /// Content-Transfer-Encoding: 7bit
    /// Approved: =?UTF-8?Q?hello_there_=F0=9F=91=8B?=
    /// X-GitHub-Recipient-Address: mlala@negabit.com
    /// X-Auto-Response-Suppress: All
    ///
    ///   Branch: refs/heads/test/rule-engine
    ///   Home:   https://github.com/viridIT/vSMTP
    ///   Commit: c6459a4946395ba90182ce7181bdbc327994c038
    ///       https://github.com/viridIT/vSMTP/commit/c6459a4946395ba90182ce7181bdbc327994c038
    ///   Author: Mathieu Lala <m.lala@viridit.com>
    ///   Date:   2022-10-26 (Wed, 26 Oct 2022)
    ///
    ///   Changed paths:
    ///     M src/vsmtp/vsmtp-rule-engine/src/api/message.rs
    ///     M src/vsmtp/vsmtp-rule-engine/src/lib.rs
    ///     M src/vsmtp/vsmtp-test/src/vsl.rs
    ///
    ///   Log Message:
    ///   -----------
    ///   test: add test on message
    ///
    ///
    /// "#;
    /// # let msg = vsmtp_mail_parser::MessageBody::try_from(msg[1..].replace("\n", "\r\n").as_str()).unwrap();
    ///
    /// # let rules = r#"
    /// #{
    ///     preq: [
    ///         rule "verify dkim" || {
    ///             dkim::verify();
    ///
    ///             // The dkim header should indicate a pass.
    ///             if !msg::get_header("Authentication-Results").contains("dkim=pass") {
    ///               return state::deny();
    ///             }
    ///
    ///             // the result of dkim verification is cached, so this call will
    ///             // not recompute the signature and recreate a header.
    ///             dkim::verify();
    ///
    ///             // FIXME: should be one.
    ///             if msg::count_header("Authentication-Results") != 2 {
    ///               return state::deny();
    ///             }
    ///
    ///             state::accept()
    ///         }
    ///    ]
    ///  }
    /// # "#;
    /// # let states = vsmtp_test::vsl::run_with_msg(|builder| Ok(builder
    /// #   .add_root_filter_rules("#{}")?
    /// #      .add_domain_rules("testserver.com".parse().unwrap())
    /// #        .with_incoming(rules)?
    /// #        .with_outgoing(rules)?
    /// #        .with_internal(rules)?
    /// #      .build()
    /// #   .build()), Some(msg));
    /// # use vsmtp_common::{status::Status, CodeID};
    /// # use vsmtp_rule_engine::ExecutionStage;
    /// # assert_eq!(states[&ExecutionStage::PreQ].2, Status::Accept(either::Left(CodeID::Ok)));
    /// ```
    ///
    /// Changing the header `Subject` will result in a dkim verification failure.
    ///
    /// ```
    /// // The message received.
    /// let msg = r#"
    /// Received: from github.com (hubbernetes-node-54a15d2.ash1-iad.github.net [10.56.202.84])
    /// 	by smtp.github.com (Postfix) with ESMTPA id 19FB45E0B6B
    /// 	for <mlala@negabit.com>; Wed, 26 Oct 2022 14:30:51 -0700 (PDT)
    /// DKIM-Signature: v=1; a=rsa-sha256; c=relaxed/relaxed; d=github.com;
    /// 	s=pf2014; t=1666819851;
    /// 	bh=7gTTczemS/Aahap1SpEnunm4pAPNuUIg7fUzwEx0QUA=;
    /// 	h=Date:From:To:Subject:From;
    /// 	b=eAufMk7uj4R+bO5Nr4DymffdGdbrJNza1+eykatgZED6tBBcMidkMiLSnP8FyVCS9
    /// 	 /GSlXME6/YffAXg4JEBr2lN3PuLIf94S86U3VckuoQQQe1LPtHlnGW5ZwJgi6DjrzT
    /// 	 klht/6Pn1w3a2jdNSDccWhk5qlSOQX9JKnE7UD58=
    /// Date: Wed, 26 Oct 2022 14:30:51 -0700
    /// From: Mathieu Lala <noreply@github.com>
    /// To: mlala@negabit.com
    /// Message-ID: <viridIT/vSMTP/push/refs/heads/test/rule-engine/000000-c6459a@github.com>
    /// Subject: Changing the header produce an invalid dkim verification
    /// Mime-Version: 1.0
    /// Content-Type: text/plain;
    ///  charset=UTF-8
    /// Content-Transfer-Encoding: 7bit
    /// Approved: =?UTF-8?Q?hello_there_=F0=9F=91=8B?=
    /// X-GitHub-Recipient-Address: mlala@negabit.com
    /// X-Auto-Response-Suppress: All
    ///
    ///   Branch: refs/heads/test/rule-engine
    ///   Home:   https://github.com/viridIT/vSMTP
    ///   Commit: c6459a4946395ba90182ce7181bdbc327994c038
    ///       https://github.com/viridIT/vSMTP/commit/c6459a4946395ba90182ce7181bdbc327994c038
    ///   Author: Mathieu Lala <m.lala@viridit.com>
    ///   Date:   2022-10-26 (Wed, 26 Oct 2022)
    ///
    ///   Changed paths:
    ///     M src/vsmtp/vsmtp-rule-engine/src/api/message.rs
    ///     M src/vsmtp/vsmtp-rule-engine/src/lib.rs
    ///     M src/vsmtp/vsmtp-test/src/vsl.rs
    ///
    ///   Log Message:
    ///   -----------
    ///   test: add test on message
    ///
    ///
    /// "#;
    /// # let msg = vsmtp_mail_parser::MessageBody::try_from(msg[1..].replace("\n", "\r\n").as_str()).unwrap();
    ///
    /// # let rules = r#"#{
    ///     preq: [
    ///         rule "verify dkim" || {
    ///             dkim::verify();
    ///
    ///             if !msg::get_header("Authentication-Results").contains("dkim=fail") {
    ///               return state::deny();
    ///             }
    ///
    ///             state::accept();
    ///         }
    ///     ]
    /// }
    /// # "#;
    /// # let states = vsmtp_test::vsl::run_with_msg(|builder| Ok(builder
    /// #   .add_root_filter_rules("#{}")?
    /// #      .add_domain_rules("testserver.com".parse().unwrap())
    /// #        .with_incoming(rules)?
    /// #        .with_outgoing(rules)?
    /// #        .with_internal(rules)?
    /// #      .build()
    /// #   .build()), Some(msg));
    /// # use vsmtp_common::{status::Status, CodeID};
    /// # use vsmtp_rule_engine::ExecutionStage;
    /// # assert_eq!(states[&ExecutionStage::PreQ].2, Status::Accept(either::Left(CodeID::Ok)));
    /// ```
    ///
    /// # rhai-autodocs:index:7
    #[rhai_fn(return_raw)]
    pub fn verify(ncc: NativeCallContext) -> EngineResult<rhai::Map> {
        let ctx = get_global!(ncc, ctx)?;
        let msg = get_global!(ncc, msg)?;
        let result = super::Impl::verify_inner(
            &ctx,
            &msg,
            &get_global!(ncc, srv)?,
            5,
            // the dns query may result in multiple public key, the registry with invalid format are ignored.
            // among ["first_one", "cycle"]
            "cycle",
            // is the `expire_time` of the signature over `now +/- epsilon` (as seconds)
            100,
        )?;

        let header_value = format!(
            r#"{};
 dkim={}`"#,
            crate::api::utils::get_root_domain(
                &vsl_guard_ok!(ctx.read()).server_name().to_string()
            ),
            result
                .get("status")
                .map(std::string::ToString::to_string)
                .unwrap_or_default()
        );

        crate::api::message::Impl::prepend_header(&msg, "Authentication-Results", &header_value)?;

        Ok(result)
    }

    /// Produce a `DKIM-Signature` header.
    ///
    /// # Args
    ///
    /// * `selector`         - the DNS selector to expose the public key & for the verifier
    /// * `private_key`      - the private key to sign the mail,
    ///                        associated with the public key in the `selector._domainkey.sdid`
    ///                        DNS record.
    /// * `headers_field`    - list of headers to sign.
    /// * `canonicalization` - the canonicalization algorithm to use. (ex: "simple/relaxed")
    ///
    /// # Effective smtp stage
    ///
    /// `preq` and onwards.
    ///
    /// # Example
    ///
    /// ```
    /// # let rules = r#"#{
    ///   preq: [
    ///     action "sign dkim" || {
    ///       for private_key in dkim::get_private_keys("testserver.com") {
    ///         dkim::sign(#{
    ///            // default: server_name()
    ///            sdid:                "testserver.com",
    ///
    ///            // mandatory
    ///            selector:            "2022-09",
    ///
    ///            // mandatory
    ///            private_key:         private_key,
    ///
    ///            // default: ["From", "To", "Date", "Subject", "From"]
    ///            headers:             ["From", "To", "Date", "Subject", "From"],
    ///
    ///            // default: "simple/relaxed"
    ///            canonicalization:    "simple/relaxed"
    ///         });
    ///       }
    ///     },
    /// #   rule "trailing" || state::accept(),
    ///   ]
    /// }
    /// # "#;
    ///
    /// # let states = vsmtp_test::vsl::run(|builder| Ok(builder
    /// #   .add_root_filter_rules("#{}")?
    /// #      .add_domain_rules("testserver.com".parse().unwrap())
    /// #        .with_incoming(rules)?
    /// #        .with_outgoing(rules)?
    /// #        .with_internal(rules)?
    /// #      .build()
    /// #   .build()));
    /// # use vsmtp_common::{status::Status, CodeID};
    /// # use vsmtp_rule_engine::ExecutionStage;
    /// # assert_eq!(states[&ExecutionStage::PreQ].2, Status::Accept(either::Left(CodeID::Ok)));
    /// ```
    ///
    /// # rhai-autodocs:index:8
    #[rhai_fn(name = "sign", return_raw)]
    pub fn sign(ncc: NativeCallContext, params: rhai::Map) -> EngineResult<()> {
        let signature = vsl_generic_ok!(super::Impl::generate_signature(
            &*vsl_guard_ok!(get_global!(ncc, msg)?.read()),
            &*vsl_guard_ok!(get_global!(ncc, ctx)?.read()),
            rhai::serde::from_dynamic::<SignatureParams>(&params.into())?
        ));

        crate::api::message::prepend_header(ncc, "DKIM-Signature", &signature)
    }
}

///
#[derive(Debug)]
pub struct DnsError(trust_dns_resolver::error::ResolveError);

impl Default for DnsError {
    fn default() -> Self {
        Self(trust_dns_resolver::error::ResolveError::from(
            trust_dns_resolver::error::ResolveErrorKind::Message("`default` invoked"),
        ))
    }
}

impl std::fmt::Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

///
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, strum::EnumMessage, strum::EnumIter, thiserror::Error)]
pub enum DkimErrors {
    ///
    #[strum(message = "neutral", detailed_message = "signature_parsing_failed")]
    #[error("the parsing of the signature failed: `{inner}`")]
    SignatureParsingFailed {
        ///
        inner: <backend::Signature as std::str::FromStr>::Err,
    },
    ///
    #[strum(message = "neutral", detailed_message = "key_parsing_failed")]
    #[error("the parsing of the public key failed: `{inner}`")]
    KeyParsingFailed {
        ///
        inner: <backend::PublicKey as std::str::FromStr>::Err,
    },
    ///
    #[strum(message = "neutral", detailed_message = "invalid_argument")]
    #[error("invalid argument: `{inner}`")]
    InvalidArgument {
        ///
        inner: String,
    },
    ///
    #[strum(message = "temperror", detailed_message = "temp_dns_error")]
    #[error("temporary dns error: `{inner}`")]
    TempDnsError {
        ///
        inner: DnsError,
    },
    ///
    #[strum(message = "permerror", detailed_message = "perm_dns_error")]
    #[error("permanent dns error: `{inner}`")]
    PermDnsError {
        ///
        inner: DnsError,
    },
    ///
    #[strum(message = "fail", detailed_message = "signature_mismatch")]
    #[error("the signature does not match: `{inner}`")]
    SignatureMismatch {
        ///
        inner: backend::VerifierError,
    },
}

impl From<DkimErrors> for Box<rhai::EvalAltResult> {
    fn from(this: DkimErrors) -> Self {
        Self::new(rhai::EvalAltResult::ErrorRuntime(
            rhai::Dynamic::from_map(rhai::Map::from_iter([
                (
                    "type".into(),
                    strum::EnumMessage::get_detailed_message(&this)
                        .expect("`DkimErrors` must have a `detailed message` for each variant")
                        .to_string()
                        .into(),
                ),
                ("inner".into(), rhai::Dynamic::from(this.to_string())),
            ])),
            rhai::Position::NONE,
        ))
    }
}

pub(super) struct Impl;

impl Impl {
    pub fn has_dkim_result(ctx: &Context) -> EngineResult<bool> {
        Ok(vsl_guard_ok!(ctx.read())
            .dkim()
            .map_err::<Box<rhai::EvalAltResult>, _>(|_| "bad state".into())?
            .is_some())
    }

    pub fn dkim_result(ctx: &Context) -> EngineResult<rhai::Map> {
        vsl_guard_ok!(ctx.read())
            .dkim()
            .map_err::<Box<rhai::EvalAltResult>, _>(|_| "bad state".into())?
            .map_or_else(
                || Err("no `dkim_result` available".into()),
                |dkim_result| {
                    Ok(rhai::Map::from_iter([(
                        "status".into(),
                        dkim_result.status.clone().into(),
                    )]))
                },
            )
    }

    #[tracing::instrument(ret, err)]
    pub fn parse_signature(input: &str) -> Result<backend::Signature, DkimErrors> {
        <backend::Signature as std::str::FromStr>::from_str(input)
            .map_err(|inner| DkimErrors::SignatureParsingFailed { inner })
    }

    #[tracing::instrument(ret, err)]
    fn verify(
        message: &MessageBody,
        signature: &backend::Signature,
        key: &backend::PublicKey,
    ) -> Result<(), DkimErrors> {
        backend::verify(signature, message.inner(), key)
            .map_err(|inner| DkimErrors::SignatureMismatch { inner })
    }

    #[tracing::instrument(skip(server), ret, err)]
    fn get_public_key(
        server: &Server,
        signature: &backend::Signature,
        on_multiple_key_records: &str,
    ) -> Result<Vec<backend::PublicKey>, DkimErrors> {
        const VALID_POLICY: [&str; 2] = ["first", "cycle"];
        if !VALID_POLICY.contains(&on_multiple_key_records) {
            return Err(DkimErrors::InvalidArgument {
                inner: format!(
                    "expected values in `[first, cycle]` but got `{on_multiple_key_records}`",
                ),
            });
        }

        let resolver = server.resolvers.get_resolver_root();

        let txt_record =
            block_on!(resolver.txt_lookup(signature.get_dns_query())).map_err(|e| {
                use trust_dns_resolver::error::ResolveErrorKind;
                if matches!(
                    e.kind(),
                    ResolveErrorKind::Message(_)
                        | ResolveErrorKind::Msg(_)
                        | ResolveErrorKind::NoConnections
                        | ResolveErrorKind::NoRecordsFound { .. }
                ) {
                    DkimErrors::PermDnsError { inner: DnsError(e) }
                } else {
                    DkimErrors::TempDnsError { inner: DnsError(e) }
                }
            })?;

        let keys = txt_record
            .into_iter()
            .map(|i| <backend::PublicKey as std::str::FromStr>::from_str(&i.to_string()));

        let keys = keys
            .collect::<Result<Vec<_>, <backend::PublicKey as std::str::FromStr>::Err>>()
            .map_err(|inner| DkimErrors::KeyParsingFailed { inner })?;

        Ok(if on_multiple_key_records == "first" {
            keys.into_iter().next().map_or_else(Vec::new, |i| vec![i])
        } else {
            keys
        })
    }

    #[tracing::instrument(ret, err)]
    fn generate_signature(
        message: &MessageBody,
        ctx: &vsmtp_common::Context,
        params: SignatureParams,
    ) -> Result<String, DkimErrors> {
        let signature = backend::sign(
            message.inner(),
            &params.private_key,
            params.sdid.unwrap_or_else(|| ctx.server_name().to_string()),
            params.selector.to_string(),
            params
                .canonicalization
                .unwrap_or_else(|| "simple/relaxed".parse().expect("default values are valid")),
            params.headers_field.unwrap_or_else(|| {
                ["From", "To", "Date", "Subject", "From"]
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            }),
        )
        .map_err(|e| DkimErrors::InvalidArgument {
            inner: format!("the signature failed: `{e}`"),
        })?;

        Ok(signature.get_signature_value())
    }

    pub fn store(ctx: &Context, result: &rhai::Map) -> EngineResult<()> {
        let result = backend::VerificationResult {
            status: result
                .get("status")
                .ok_or_else::<Box<rhai::EvalAltResult>, _>(|| {
                    "`status` is missing in DKIM verification result".into()
                })?
                .to_string(),
        };

        Ok(vsl_generic_ok!(vsl_guard_ok!(ctx.write()).set_dkim(result)))
    }

    pub fn verify_inner(
        ctx: &Context,
        msg: &Message,
        srv: &Server,
        nbr_headers: usize,
        on_multiple_key_records: &str,
        expiration_epsilon: u64,
    ) -> EngineResult<rhai::Map> {
        if Self::has_dkim_result(ctx)? {
            Self::dkim_result(ctx)
        } else {
            let result = Self::verify_first_signature_or_error(
                msg,
                srv,
                nbr_headers,
                on_multiple_key_records,
                expiration_epsilon,
            )?;
            Self::store(ctx, &result)?;

            Ok(result)
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn verify_first_signature_or_error(
        msg: &Message,
        srv: &Server,
        nbr_headers: usize,
        on_multiple_key_records: &str,
        expiration_epsilon: u64,
    ) -> EngineResult<rhai::Map> {
        tracing::debug!(%nbr_headers, %on_multiple_key_records, %expiration_epsilon, "Verifying DKIM signature.");

        let mut last_error: Option<String> = None;

        let mut header = crate::api::message::Impl::get_header_untouched(msg, "DKIM-Signature")?;
        header.truncate(nbr_headers);

        for input in header {
            let signature = match Self::parse_signature(&input.to_string()) {
                Ok(signature) => signature,
                Err(error) => {
                    tracing::warn!(%error, "Failed to parse DKIM signature, continuing ...");
                    last_error = Some(Self::get_dkim_error_status(&error));
                    continue;
                }
            };

            if signature.has_expired(expiration_epsilon) {
                tracing::warn!("DKIM signature expired, continuing ...");
                continue;
            }

            // NOTE: for any reason, you can decide to ignore the signature
            // if signature... {
            //     continue;
            // }

            for key in &Self::get_public_key(srv, &signature, on_multiple_key_records)? {
                if let Err(error) = Self::verify(&*vsl_guard_ok!(msg.read()), &signature, key) {
                    tracing::warn!(%error, "DKIM signature verification failed");
                    last_error = Some(Self::get_dkim_error_status(&error));
                    continue;
                }

                tracing::debug!("DKIM signature successfully verified.");

                if key.has_debug_flag() {
                    tracing::warn!("DKIM signature contains `debug_flag`, continuing");
                    continue;
                }

                // header.b & header.a can be set optionally
                return Ok(rhai::Map::from_iter([
                    ("status".into(), "pass".into()),
                    ("sdid".into(), signature.sdid.into()),
                    ("auid".into(), signature.auid.into()),
                ]));
            }
        }

        tracing::warn!("no valid DKIM signature");

        Ok(rhai::Map::from_iter([(
            "status".into(),
            last_error.unwrap_or_else(|| "none".to_string()).into(),
        )]))
    }

    fn get_dkim_error_status(error: &DkimErrors) -> String {
        strum::EnumMessage::get_message(error)
            .expect("`DkimErrors` must have a `message` for each variant")
            .to_string()
    }
}
