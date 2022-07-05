/*
 Copyright (c) 2022 ParallelChain Lab
 
 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.
 
 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.
 
 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

/// `Error` is the error type returned for failure in deserialzation process
#[derive(Debug)]
pub struct Error (ErrorKind);

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ErrorKind {
    IncorrectLength,
    ReceiptStatusCodeOutOfRange,
    StringParseError,
}

impl Error {
    pub fn new(errorkind: ErrorKind) -> Self {
        Self(errorkind)
    }

    pub fn kind(&self) -> ErrorKind {
        self.0
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Deserialization error"
    }
}
