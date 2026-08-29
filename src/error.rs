// Copyright 2026 ZenLunarDev
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error type for `ratatui-plus`.

use std::fmt;

/// Convenience result alias used across the crate.
pub type Rt<T = (), E = Error> = Result<T, E>;

/// Crate-wide error type.
#[derive(Debug)]
pub enum Error {
    /// Wraps a [`std::io::Error`].
    Io(std::io::Error),
    /// Terminal setup / control failure.
    Term(String),
    /// A formatting error bubbled up from `write!`.
    Fmt(fmt::Error),
    /// Generic failure with a message.
    Other(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Self {
        Error::Fmt(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::Term(e) => write!(f, "terminal error: {e}"),
            Error::Fmt(e) => write!(f, "format error: {e}"),
            Error::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

/// Build an [`Error::Other`] from anything displayable.
pub fn err<T: fmt::Display>(m: T) -> Error {
    Error::Other(m.to_string())
}
