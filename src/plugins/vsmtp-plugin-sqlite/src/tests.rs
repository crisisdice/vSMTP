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

use crate::api::sqlite_api;
use rhai::Engine;

#[test]
fn test_query() {
    let engine = Engine::new();
    let map = engine.parse_json(
        r#"
            {
                "path": "sharks.db",
                "connections": 1,
                "timeout": "1s"
            }"#,
        true,
    );
    let mut server = sqlite_api::connect(map.unwrap()).unwrap();
    sqlite_api::query_str(&mut server, "CREATE TABLE sharks(id integer NOT NULL, name text NOT NULL, sharktype text NOT NULL, length integer NOT NULL);").unwrap();
    sqlite_api::query_str(&mut server, "INSERT INTO sharks VALUES (1, \"Sammy\", \"Greenland Shark\", 427);").unwrap();
    sqlite_api::query_str(&mut server, "INSERT INTO sharks VALUES (2, \"Alyoshka\", \"Great White Shark\", 600);").unwrap();
    sqlite_api::query_str(&mut server, "INSERT INTO sharks VALUES (3, \"Himari\", \"Megaladon\", 1800);").unwrap();
    sqlite_api::query_str(&mut server, "SELECT * FROM sharks;").unwrap();
    dbg!(sqlite_api::query_str(&mut server, "SELECT * FROM sharks;")).unwrap();
}
