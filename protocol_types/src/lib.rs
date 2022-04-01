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

//! library crate `protocol-types` uses Protocol Buffers to define common, language agnostic types that are
//! part of the core semantics of Mainnet (e.g., Transactions).
//! 
//! PROST! (a Rust implementation of Protocol Buffers) allow users of this crate to easily deserialize these
//! wire types into easy-to-use Rust types, and vice versa: convert these Rust-types into vectors of bytes
//! transportable over the wire.
//! 
//! run `cargo doc --open` to view rich documentation on the available types (these are difficult to view
//! in an IDE, since the prost_build process is opaque and inclusion of the generated code is through an
//! `include!` macro).



// Cryptography related protocol types, including public addresses, secret keys, signatures, and hashes.
pub mod crypto;

// Re-exports
pub use sc_params::*;
pub use crypto::*;
pub use transaction::*;

pub mod sc_params {
    use std::io::Cursor;

    // message exposes default trait impls for encoding and decoding message types.
    use prost::Message;

    // copy `protoc` generated code for the sc_params package.
    include!(concat!(env!("OUT_DIR"), "/sc_params.rs"));

    impl Serializable<ParamsFromTransaction> for ParamsFromTransaction {}
    impl Deserializable<ParamsFromTransaction> for ParamsFromTransaction {}

    impl Serializable<ParamsFromBlockchain> for ParamsFromBlockchain {}
    impl Deserializable<ParamsFromBlockchain> for ParamsFromBlockchain {}

    pub trait Serializable<T: Message> {
        fn serialize(msg: &T) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.reserve(msg.encoded_len());
            msg.encode(&mut buf).unwrap();

            buf
        }
    }

    pub trait Deserializable<T: Message + std::default::Default> {
        fn deserialize(buf: &[u8]) -> Result<T, prost::DecodeError> {
            T::decode(&mut Cursor::new(buf))
        }
    }
}
pub mod transaction {
    use std::io::Cursor;

    // message exposes default trait impls for encoding and decoding message types,
    // e.g. transaction::Transction.
    use prost::Message;

    // copy `protoc` generated code for the transaction package.
    include!(concat!(env!("OUT_DIR"), "/transaction.rs"));

    impl Serializable<Block> for Block {}
    impl Deserializable<Block> for Block {}

    impl Serializable<Blocks> for Blocks {}
    impl Deserializable<Blocks> for Blocks {}

    impl Serializable<BlockHeader> for BlockHeader {}
    impl Deserializable<BlockHeader> for BlockHeader {}

    impl Serializable<Transaction> for Transaction {}
    impl Deserializable<Transaction> for Transaction {}

    impl Serializable<Transactions> for Transactions {}
    impl Deserializable<Transactions> for Transactions {}

    impl Serializable<Event> for Event {}
    impl Deserializable<Event> for Event {}

    impl Serializable<Receipt> for Receipt {}
    impl Deserializable<Receipt> for Receipt {}

    pub trait Serializable<T: Message> {
        fn serialize(msg: &T) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.reserve(msg.encoded_len());
            msg.encode(&mut buf).unwrap();

            buf
        }
    }

    pub trait Deserializable<T: Message + std::default::Default> {
        fn deserialize(buf: &[u8]) -> Result<T, prost::DecodeError> {
            T::decode(&mut Cursor::new(buf))
        }
    }
}
