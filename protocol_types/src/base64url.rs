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

use std::ops::Deref;

use base64;

/// Base64URL are Strings restricted to containing the 2^6 UTF-8 code points in the Base64URL bytes-to-characters encoding.
/// Base64URL MUST NOT contain padding. 
pub struct Base64URL(String);


impl Base64URL {
    /// encode takes in a slice of bytes and returns the bytes encoded as a Base64URL String. 
    pub fn encode<T: AsRef<[u8]>>(bytes: T) -> Base64URL { 
        Base64URL(base64::encode_config(bytes, base64::Config::new(base64::CharacterSet::UrlSafe, false)))
    }

    /// decode takes in a string and tries to decode it into a Vector of bytes. It returns a base64::DecodeError if `string`
    /// is not valid Base64URL.
    pub fn decode<T: ?Sized + AsRef<[u8]>>(base64_url: &T) -> Result<Vec<u8>, base64::DecodeError> {
        base64::decode_config(base64_url, base64::Config::new(base64::CharacterSet::UrlSafe, false))
    } 
}


impl Deref for Base64URL {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}
