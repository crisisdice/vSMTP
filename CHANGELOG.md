# Changelog

All notable changes are documented or linked to in this file. The format of the changelog is based on
['Keep a Changelog'](https://keepachangelog.com/en/1.0.0/). In short, this changelog is sorted the most recent
release at the top, and the first section documents features that are in the `development`
branch but have not yet landed in the `main` branch from which releases are generated.

The MSRV (minimum supported Rust version) of the project is documented in the [`Cargo.toml`](Cargo.toml) and in the
[README](README.md). Changes to the MSRV are considered a **non-breaking change** and thus can happen in a *MINOR*
release. They will however *never* happen in a patch release.

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [2.2.0] - 2023-03-19

### BREAKING CHANGES

* the `spf::check` function takes a map of parameters. (#1079)

```js
spf::check(#{ header: "spf", policy: "soft" })
```

### Added

* `transport::forward` parameters can be passed as url. (#1018)

```js
#{
  action "forward to my server" || {
    let user = "foo";
    let pass = "bar";
    let host = "smtp.mydomain.tld";
    let port = 25;

    transport::forward_all(`smtp://${user}:${pass}@${host}:${port}`);
    // or simply
    transport::forward_all(env("MY_VAR"));


    // you can configure the tls policy:
    transport::forward_all("smtp://[::1]?tls=opportunistic");
    // `tls` value being among [none, opportunistic, required, tunnel]

    transport::forward_all("smtps://domain.tld");
    // is equivalent to
    transport::forward_all("smtp://domain.tld?tls=tunnel");
  }
}
```

* A LDAP plugin, enabling you to search and compare attributes in an active directory. (#928)

```js
import "plugins/libvsmtp_plugin_ldap" as ldap;

// A service used to connect to and query an active directory.
export const directory = ldap::connect(#{
  url: "ldap://openldap:1389",
  connections: 4,
  timeout: "20s",
  bind: #{
    dn: "cn=admin,dc=example,dc=org",
    pw: "admin",
  }
});
```

```js
#{
  mail: [
    rule "search for user in AD" || {
      let user = ctx::mail_from().local_part;
      let address = ctx::mail_from().to_string();

      let search = directory.search(
          "dc=example,dc=org",
          // Search the whole tree.
          "sub",
          // Match on the user id and address.
          `(|(uid=${user})(mail=${address}))`,
          // Get all attributes from the entries.
          []
      );

      if search.result == "error" {
          log("warn", `User could not be found in database: ${search.error}`);
          return state::deny();
      }

      log("info", `Entry for ${user} found in AD.`);

      // Log every entry and their attributes.
      for entry in search.entries {
          log("info", `dn=${entry.dn}`);
          log("info", `dn=${entry.attrs}`);
          for attr in entry.attrs.keys() {
              for value in entry.attrs.get(attr) {
                  log("info", `  attr=${attr},value=${value}`);
              }
          }
      }

      state::next()
    }
  ]
}
```

* `config.server.tls.certificate` & `config.server.tls.private_key` (optional) used when a SNI is not provided or not found among the virtuals server

### Modified

* vSMTP loads `/etc/vsmtp/vsmtp.vsl` by default if `-c` flag is missing, and stops if no configuration could be found in this path. (#1020)
* Better Rhai API documentation. (#1079)

### Fixed

* `deliver` (default transport method) will accept the CN correctly (#1018)

  Before a MX `mta-01.smtp.mydomain.tld` in the dns zone of `mydomain.tld` must had a certificate with the CN being `mydomain.tld`.
  Now `mta-01.smtp.mydomain.tld`, `smtp.mydomain.tld` or `mydomain.tld` will be accepted.

* `action` completely ignores it's return value. (#1024)
* `spf::check` can be used from the `mail` stage. (#1027)

## Removed

* the `is_default` properties in the `on_domain_config()`
* the assignation of a default vritual server using a symlink named `default`

## [2.1.1] - 2023-02-10

### Modified

* `dkim::sign()` prototype changed, taking a map of arguments instead of multiple arguments. (#1008)
* TLS connection will be served if the client do not use a SNI, and a `default` domain is configured. (#1005)

### Fixed

* `srv()` variable can be accessed outside rules. (#1007)

### Documented

* `cmd` and `smtp` vsl modules. (#1004)

## [2.1.0] - 2023-02-01

### Added

* A public [memcached](https://www.memcached.org/) plugin. (#974)
* A `env` function, enable fetching environment variables in `vsl` scripts. (#927)
  exported globally and available in the `unix` module.

```rust
fn on_config(config) {

  config.app.vsl.filter_path = env("VSMTP_FILTER_PATH");

  config
}
```

* The `user_exist`, `env` and `hostname` functions are available in the config scripts. (#927)
* Support for fqdn in the `config.server.interfaces` `addr`, `addr_submission` and `addr_submissions` fields. (#965)

```rust
fn on_config(config) {

  config.server.interfaces = #{
      addr: ["example.com:25", "localhost:10025"],
      // ...
  };

  config
}
```

* Documentation for docker images. (#968)

### Fixed

* Display proper configuration error messages on machine that do not have a 'vsmtp' user. (#926)
* Create proper build systems to share debian and ubuntu packages. (#933)
* Building without `.git` no longer causes a hard failure. (#952)

### Changed

* Changes for vSMTP's official docker image.
  * Allow specifying vSMTP's branch. (will be used to make an "unstable" tag)
  * Install missing dependencies required by internal dependencies updates.
  * Automatically import all plugins into the image.
  * Symlink all plugins to `/etc/vsmtp/plugins` by default.
  * Output vSMTP's current version number.
  * Set default command to a vsmtp without daemon mode.

### Removed

* `info()` and `info(code)` vsl api function (#972)

### Compatibility Notes

* bumped MSRV to 1.66.1 (#976)

## [2.0.0] - 2023-01-09

### Plugin System

To extend the functionality of vSMTP, we have added a plugin system. You will be able to choose plugins you
are interested in by importing them in your vSL script.

Plugins are implemented as dynamic libraries, and are imported in rhai scripts using
[`rhai-dylib`](https://crates.io/crates/rhai-dylib). (#753)

Example:

```rust
// import the dynamic library in Rhai.
import "/usr/lib/vsmtp/libvsmtp-plugin-csv" as db;

// use functions defined in the library.
db::csv(#{ ... });
```

Implementing csv and mysql databases as a plugins. (#625)

### Configuration in vSL

Previous configurations were written in TOML, now they are written in vSL. (#685)

```rust
fn on_config(config) {
  config.version_requirement = ">=2.0.0, <3.0.0";

  config.server.name = "my.fqdn.com";

  config.server.system = #{
      user: "root",
      group: "root",
  };

  config.server.interfaces = #{
      addr: ["127.0.0.1:25"],
      addr_submission: ["127.0.0.1:587"],
      addr_submissions: ["127.0.0.1:465"],
  };

  config
}
```

The `toml` vsl module has been renamed to `cfg`. (#709)

### Filtering enhancement

* The policy execution has changed, it depends on the virtual domain
and the transaction types (incoming, outgoing, internal). (#709)

```
/etc/vsmtp
┣ vsmtp.vsl
┣ conf.d/
┃     ┣ config.vsl
┃     ┣ interfaces.vsl
┃     ┣ logs.vsl
┃     ┗ other.vsl
┣ domain-available/
┃     ┣ main.vsl            # Rules executed before the 'mail' stage
┃     ┣ fallback.vsl        # Rules executed if the domain is not handled.
┃     ┣ example.com/
┃     ┃    ┣ incoming.vsl   # Sender domain unknown, recipient domain is 'example.com'.
┃     ┃    ┣ outgoing.vsl   # Sender domain is 'example.com', recipient domain is different.
┃     ┃    ┗ internal.vsl   # Sender & recipient domain are both 'example.com'.
┃     ┗ test.com/
┃         ┗ ...
┗ domain-enabled/
      ┗ example.com -> /etc/vsmtp/domain-available/example.com
```

* Changed the API of objects to be simple rhai functions, removing implicit `export` of
  objects. (#647)

```js
// Old syntax
object localhost ip4 = "127.0.0.1";
// New syntax
const localhost = ip4("127.0.0.1");
```

* Remove Group object & function, replaced by Rhai arrays. (#660)

```js
const localhost = ip4("127.0.0.1");
const john = identifier("john.doe");

// declaration of a group.
const group = [ localhost, john ];
```

* Moved vSL syntax to a crate for better reusability. (#660)
* Remove File object, replaced by Rhai arrays. (#660)

```js
// This returns an Array of addresses.
const whitelist = file("/etc/vsmtp/whitelist.txt", "address");

for addr in whitelist {
  // ...
}
```

* Add the support of null reverse path
* A delegation cargo feature on `vsmtp-rule-engine`. (#660)

## [1.3.4] - 2022-10-20

### Fixed

* `forward` && `forward_all` functions now take port into account in socket strings. (#695)

## [1.3.3] - 2022-10-03

### Added

* `--stdout` flag, print logs to stdout. (#579)
* Message size limit configuration. (#580)
* Add the git commit hash to the version string `--version`. (#581)
* Ed25519 support. (#600)

### Changed

* MSRV bumped to 1.63.0 (#638)
* `--no-daemon` flag do not print logs to the standard output anymore. (#579)
* Refactorization of services parsing. (#576)
* `vsmtp` & `vqueue` `--version` flag display build commit. (#585)
* `vqueue` display error if no subcommands are specified. (#585)
* Updated logs to communicate better the state of vSMTP. (#587)
* Remove config field (`server.smtp.required_extension`/`app.logs.format`) and prepare for the all .vsl config.

### Unstable

* Add a `Dockerfile` for the `vsmtp`.

## [1.3.0] - 2022-09-07

### Added

* support for `MySQL` databases. see [`/examples/greylist/mysql`](https://github.com/viridIT/vSMTP/tree/develop/examples/greylist/mysql). (#548)

### Changed

* update the backend of the `SASL` protocol, using a state-of-the-art Rust implementation `rsasl` instead of binding the  `gsasl` C library. (#536)
* update the vsl api with more consistent syntax. (#545)
* Greylist sender domain & return code. (#566, #571)

### Fixed

* IPv6 address for `EHLO` command. (#530)
* Log level ordering. (#565)

### Documented

* improve the vsl api documentation. (#545, #553)

## [1.2.1] - 2022-08-26

### Added

* `check_dmarc` vsl function. (#506)
* Syslog configuration. (#509)
* journald support (#482)

### Fixed

* Missing documentation for vsl api. (#503, #513, #518)
* Don't send greeting code right after receiving STARTTLS. (#504)
* Initialize logs before privilege drop. (#506)
* Documentation errors in Readme and other files.

## [1.2.0] - 2022-08-12

### Added

* "Deliver-To" header to local delivery (mbox & maildir) (#443)
* `lookup` & `rlookup` vsl functions. (#473)
* Support for DKIM. (#457)
* Support for syslogs. (#475)

### Changed

* Stabilization of vsl's api. (#452)
* Replaced `log4rs` by the `tracing` crate for logs. (#460)

## [1.1.3] - 2022-07-12

### Changed

* delegation directives sets the X-VSMTP-DELEGATION
  header in the email to pick up where processing left of. (#438)
* refactored the queue system to make it simpler. (#438)
* multiple delegation directives can be used in a single
  stage with one or multiple services. (#438)
* delegation is available for `postq` & `delivery` only. (#438)

## [1.1.2] - 2022-07-07

### Added

* a `prepend_header` and `append_header` in `vsl` api to push front/back headers in message (#410).
* run the deliveries of message concurrently (by transport method maildir/mbox/...) (#425).

### Changed

* you can now add headers to a message at any stages (instead of preq an later) (#410).

### Fixed

* improve SPF policies (#400).
* produce an error at startup with invalid rules stages (#391).
* fixed a bug where the delivery system would place successfully sent emails into deferred queue when only one MX record was available (#427).

### Breaking Changes

* `vsl` api of SPF has changed (see the documentation <https://vsmtp.rs/start/configuration/hardening.html#using-the-spf-protocol>).
* split `check_relay` to `check_mail_relay` and `check_rcpt_relay` (#412).

## [1.1.0] - 2022-06-20
