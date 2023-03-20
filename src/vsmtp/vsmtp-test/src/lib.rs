//! vSMTP testing utilities

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

#![doc(html_no_source)]
#![deny(missing_docs)]
//
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

///
#[macro_export]
macro_rules! arc {
    (  $x:expr  ) => {
        std::sync::Arc::new($x)
    };
}

/// Config shortcut
pub mod config;

///
pub mod receiver;
mod recv_handler_wrapper;
pub use recv_handler_wrapper::Wrapper;

///
pub mod get_tls_file;

///
pub mod vsl;

#[cfg(test)]
mod tests;
