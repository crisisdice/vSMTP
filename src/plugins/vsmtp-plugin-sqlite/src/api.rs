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

use std::str::FromStr;

use rusqlite::Result;

use rhai::plugin::{
    mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

/// Parameters available for the sqlite service. Used
/// with serde for easy parsing.
#[derive(Debug, serde::Deserialize)]
struct SQLiteDatabaseParameters {
    pub path: String,
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: std::time::Duration,
    #[serde(default = "default_connections")]
    pub connections: rhai::INT,
}

const fn default_connections() -> rhai::INT {
    4
}

const fn default_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(30)
}

/// A r2d2 connection manager for sqlite.
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ConnectionManager {
    path: String,
}

impl ConnectionManager {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl r2d2::ManageConnection for ConnectionManager {
    type Connection = rusqlite::Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<rusqlite::Connection, rusqlite::Error> {
        rusqlite::Connection::open(self.path.clone())
    }

    fn is_valid(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        rusqlite::Connection::query_row(conn, "SELECT sqlite_version()", (), |row| row.get(0))
            .map(|_: String| ())
    }

    fn has_broken(&self, conn: &mut rusqlite::Connection) -> bool {
        self.is_valid(conn).is_err()
    }
}

/// A database connector based on sqlite.
#[derive(Debug, Clone)]
pub struct SQLiteConnector {
    /// The url to the database.
    pub url: String,
    /// connection pool for the database.
    pub pool: r2d2::Pool<ConnectionManager>,
}

#[derive(Debug)]
struct Wrapper(Dynamic);

impl rusqlite::types::FromSql for Wrapper {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match rusqlite::types::Value::from(value) {
            rusqlite::types::Value::Null => Ok(Wrapper(rhai::Dynamic::UNIT)),
            rusqlite::types::Value::Integer(v) => Ok(Wrapper(rhai::Dynamic::from_int(v))),
            rusqlite::types::Value::Real(v) => Ok(Wrapper(rhai::Dynamic::from_float(v))),
            rusqlite::types::Value::Text(v) => Ok(Wrapper(Dynamic::from(v))),
            rusqlite::types::Value::Blob(v) => Ok(Wrapper(Dynamic::from(String::from_utf8(v)))),
        }
    }
}

impl SQLiteConnector {
    pub fn query(&self, query: &str) -> Result<Vec<rhai::Map>, Box<rhai::EvalAltResult>> {
        let mut returned_rows = Vec::new();
        let client = self.pool.get();
        match client {
            Ok(client) => {
                let mut stmt = client
                    .prepare(query)
                    .map_err::<Box<rhai::EvalAltResult>, _>(|err| err.to_string().into())?;
                let column_names = stmt
                    .column_names()
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>();
                let mut rows = stmt
                    .query([])
                    .map_err::<Box<rhai::EvalAltResult>, _>(|err| err.to_string().into())?;
                loop {
                    let mut values = rhai::Map::new();
                    match rows.next() {
                        Ok(Some(row)) => {
                            let mut index = 0;
                            loop {
                                match row.get::<usize, Wrapper>(index) {
                                    Ok(value) => {
                                        let name = match column_names.get(index) {
                                            Some(column) => column,
                                            None => break,
                                        };
                                        values.insert(
                                            smartstring::SmartString::from_str(name)
                                                .map_err::<Box<rhai::EvalAltResult>, _>(|err| {
                                                    err.to_string().into()
                                                })?,
                                            value.0,
                                        );
                                        index += 1;
                                        continue;
                                    }
                                    Err(e) if e.to_string().contains("Invalid column index") => {
                                        break
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                    returned_rows.push(values);
                }
            }
            Err(e) => {
                Err(e).map_err::<Box<rhai::EvalAltResult>, _>(|err| err.to_string().into())?
            }
        }
        Ok(returned_rows)
    }
}

/// This plugin exposes methods to open a pool of connexions to a sqlite database using
/// Rhai.
#[rhai::plugin::export_module]
pub mod sqlite_api {

    pub type SQLite = rhai::Shared<SQLiteConnector>;

    /// Open a pool of connections to a SQLite database.
    ///
    /// # Args
    ///
    /// * `parameters` - a map of the following parameters:
    ///     * `path` - a string which describe the database path.
    ///     * `timeout` - time allowed between each query to the database. (default: 30s)
    ///     * `connections` - Number of connections to open to the database. (default: 4)
    ///
    /// # Return
    ///
    /// A service used to query the database pointed by the `path` parameter.
    ///
    /// # Error
    ///
    /// * The service failed to connect to the database.
    ///
    /// # Example
    ///
    /// ```text
    /// // Import the plugin stored in the `plugins` directory.
    /// import "plugins/libvsmtp_plugin_sqlite" as sqlite;
    ///
    /// export const database = sqlite::connect(#{
    ///     // Connect to a database on the system with the path "./src/plugins/vsmtp-plugin-sqlite/greylist.db"
    ///     path: "./src/plugins/vsmtp-plugin-sqlite/greylist.db",
    ///     timeout: "1m",
    ///     connections: 1,
    /// });
    /// ```
    #[rhai_fn(global, return_raw)]
    pub fn connect(parameters: rhai::Map) -> Result<SQLite, Box<rhai::EvalAltResult>> {
        let parameters = rhai::serde::from_dynamic::<SQLiteDatabaseParameters>(&parameters.into())?;

        let manager = ConnectionManager::new(parameters.path.clone());

        Ok(rhai::Shared::new(SQLiteConnector {
            url: parameters.path,
            pool: r2d2::Pool::builder()
                .max_size(
                    u32::try_from(parameters.connections)
                        .map_err::<Box<rhai::EvalAltResult>, _>(|err| err.to_string().into())?,
                )
                .connection_timeout(parameters.timeout)
                .build(manager)
                .map_err::<Box<rhai::EvalAltResult>, _>(|err| err.to_string().into())?,
        }))
    }

    /// Query the database.
    ///
    /// # Args
    ///
    /// * `query` - The query to execute.
    ///
    /// # Return
    ///
    /// A list of records.
    ///
    /// # Example
    ///
    /// Build a service in `services/database.vsl`;
    ///
    /// ```text
    /// // Import the plugin stored in the `plugins` directory.
    /// import "plugins/libvsmtp_plugin_sqlite" as sqlite;
    ///
    /// export const database = sqlite::connect(#{
    ///     // Connect to a database on the system with the path "./src/plugins/vsmtp-plugin-sqlite/greylist.db"
    ///     path: "./src/plugins/vsmtp-plugin-sqlite/greylist.db",
    ///     timeout: "1m",
    ///     connections: 1,
    /// });
    /// ```
    ///
    /// Query the database during filtering.
    ///
    /// ```text
    /// import "services/database" as srv;
    ///
    /// #{
    ///     connect: [
    ///         action "get records from my database" || {
    ///             // For the sake of this example, we assume that there is a populated
    ///             // table called 'my_table' in the database.
    ///             const records = srv::database.query("SELECT * FROM my_table");
    ///
    ///             // `records` is an array, we can run a for loop and print all records.
    ///             log("info", "fetching sqlite records ...");
    ///             for record in records {
    ///                 log("info", ` -> ${record}`);
    ///             }
    ///         }
    ///     ],
    /// }
    /// ```
    #[rhai_fn(global, name = "query", return_raw, pure)]
    pub fn query_str(
        database: &mut SQLite,
        query: &str,
    ) -> Result<rhai::Array, Box<rhai::EvalAltResult>> {
        super::query(database, query)
    }

    /// Query the database.
    ///
    /// # Args
    ///
    /// * `query` - The query to execute.
    ///
    /// # Return
    ///
    /// A list of records.
    ///
    /// # Example
    ///
    /// Build a service in `services/database.vsl`;
    ///
    /// ```text
    /// // Import the plugin stored in the `plugins` directory.
    /// import "plugins/libvsmtp_plugin_sqlite" as sqlite;
    ///
    /// export const database = sqlite::connect(#{
    ///     // Connect to a database on the system with the path "./src/plugins/vsmtp-plugin-sqlite/greylist.db"
    ///     path: "./src/plugins/vsmtp-plugin-sqlite/greylist.db",
    ///     timeout: "1m",
    ///     connections: 1,
    /// });
    /// ```
    ///
    /// Query the database during filtering.
    ///
    /// ```text
    /// import "services/database" as srv;
    ///
    /// #{
    ///     connect: [
    ///         action "get records from my database" || {
    ///             // For the sake of this example, we assume that there is a populated
    ///             // table called 'my_table' in the database.
    ///             const records = srv::database.query("SELECT * FROM my_table");
    ///
    ///             // `records` is an array, we can run a for loop and print all records.
    ///             log("info", "fetching sqlite records ...");
    ///             for record in records {
    ///                 log("info", ` -> ${record}`);
    ///             }
    ///         }
    ///     ],
    /// }
    /// ```
    #[rhai_fn(global, name = "query", return_raw, pure)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn query_obj(
        database: &mut SQLite,
        query: vsmtp_rule_engine::api::SharedObject,
    ) -> Result<rhai::Array, Box<rhai::EvalAltResult>> {
        super::query(database, &query.to_string())
    }
}

/// Query a database.
fn query(
    database: &sqlite_api::SQLite,
    query: &str,
) -> Result<rhai::Array, Box<rhai::EvalAltResult>> {
    database.query(query).map_or_else(
        |_| Ok(rhai::Array::default()),
        |record| Ok(record.into_iter().map(rhai::Dynamic::from).collect()),
    )
}
