/*
 Copyright 2022 ParallelChain Lab

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 */

use crate::{crypto, Serializable, Deserializable};


/// ParamsFromTransaction defines information that supplies to contract method exection.
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct ParamsFromTransaction {
    /// From Address of this transaction
    pub from_address :crypto::PublicAddress,
    /// To Address of the transaction
    pub to_address :crypto::PublicAddress,
    /// Transaction data. Equivalent to "data" in [crate::Transaction]
    pub data :Vec<u8>,
    /// Transaction value. Equivalent to "value" in [crate::Transaction]
    pub value :u64,
    /// Transaction hash. Equivalent to "hash" in [crate::Transaction]
    pub transaction_hash :crypto::Sha256Hash
}

/// ParamsFromBlockchain defines information that supplies to contract method exection.
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct ParamsFromBlockchain {
    /// Height of the Block
    pub this_block_number :u64,
    /// Previous Block Hash 
    pub prev_block_hash :crypto::Sha256Hash,
    /// Unix timestamp
    pub timestamp :u32,
    /// Random Bytes
    pub random_bytes :crypto::Sha256Hash,
}

/// CallData defines the data format that passes to entry point of the contact
/// 
/// The struct contains data types which are serialized into the field "data" in [crate::Transaction].
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct CallData {
    /// function name of contract with entrypoint methods. Empty string can be used for calling `init` method.
    pub method_name :String,

    /// arguments to function (entrypoint method)
    /// In contract with entrypoint methods, the arguments should be deserialized to vector of Vec<u8> and then pass as function arguments
    pub arguments :Vec<u8>
}

impl Serializable<ParamsFromTransaction> for ParamsFromTransaction {}
impl Deserializable<ParamsFromTransaction> for ParamsFromTransaction {}
impl Serializable<ParamsFromBlockchain> for ParamsFromBlockchain {}
impl Deserializable<ParamsFromBlockchain> for ParamsFromBlockchain {}
impl Serializable<CallData> for CallData {}
impl Deserializable<CallData> for CallData {}