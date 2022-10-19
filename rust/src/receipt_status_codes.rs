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

use crate::{Serializable, Deserializable};
use std::convert::TryFrom;


/// ReceiptStatusCode defines the success and error types of receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptStatusCode {

    /* Success class. */

    /// Successful transaction.
    Success,

    /* Pre-Inclusion Decision class */

    /// Nonce is not current nonce + 1 
    WrongNonce,

    /// Not enough balance to pay for gas limit.
    NotEnoughBalanceForGasLimit,

    /// Not enough balance to pay for transfer.
    NotEnoughBalanceForTransfer,

    /// Gas limit was insufficient to cover pre-execution costs.
    PreExecutionGasExhausted,

    /* Deploy class. */

    /// The contract bytecode contains disallowed opcodes.
    DisallowedOpcode,

    /// Contract cannot be compiled into machine code (it is probably invalid WASM).
    CannotCompile,

    /// Contract does not export the METHOD_CONTRACT method.
    NoExportedContractMethod,
    
    /// Deployment failed for some other reason.
    OtherDeployError,

    /* EtoC class. */

    /// Gas limit was insufficient to cover execution proper costs.
    ExecutionProperGasExhausted,

    /// Runtime error during execution proper of the entree smart contract.
    RuntimeError,

    /* Internal Transaction class. */

    /// Gas limit was insufficient to cover execution proper costs of an internal transaction.
    InternalExecutionProperGasExhaustion,

    /// Runtime error during execution proper of an internal transaction.
    InternalRuntimeError,

    /// Not enough balance to pay for transfer in an internal transaction.
    InternalNotEnoughBalanceForTransfer,

    /* Miscellaneous class. */

    /// Other error. 
    Else,
}

impl borsh::BorshSerialize for ReceiptStatusCode {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let c: u8 = self.clone().into();
        c.serialize(writer)
    }
}

impl borsh::BorshDeserialize for ReceiptStatusCode {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        match u8::deserialize(buf) {
            Ok(b) => {
                match Self::try_from(b) {
                    Ok(sc) => Ok(sc),
                    Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Cannot convert from ReceiptStatusCode"))
                }
            },
            Err(e) => Err(e),
        }
    }
}

impl Into<u8> for ReceiptStatusCode {
    fn into(self) -> u8 {
        match self {
            ReceiptStatusCode::Success => 00,

            ReceiptStatusCode::WrongNonce => 10,
            ReceiptStatusCode::NotEnoughBalanceForGasLimit => 11,
            ReceiptStatusCode::NotEnoughBalanceForTransfer => 12,
            ReceiptStatusCode::PreExecutionGasExhausted => 13,

            ReceiptStatusCode::DisallowedOpcode => 20,
            ReceiptStatusCode::CannotCompile => 21,
            ReceiptStatusCode::NoExportedContractMethod => 22,
            ReceiptStatusCode::OtherDeployError => 23,

            ReceiptStatusCode::ExecutionProperGasExhausted => 30,
            ReceiptStatusCode::RuntimeError => 31,

            ReceiptStatusCode::InternalExecutionProperGasExhaustion => 40,
            ReceiptStatusCode::InternalRuntimeError => 41,
            ReceiptStatusCode::InternalNotEnoughBalanceForTransfer => 42,

            ReceiptStatusCode::Else => 50,
        } 
    }
}

impl TryFrom<u8> for ReceiptStatusCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            00 => Ok(ReceiptStatusCode::Success),

            10 => Ok(ReceiptStatusCode::WrongNonce),
            11 => Ok(ReceiptStatusCode::NotEnoughBalanceForGasLimit),
            12 => Ok(ReceiptStatusCode::NotEnoughBalanceForTransfer),
            13 => Ok(ReceiptStatusCode::PreExecutionGasExhausted),

            20 => Ok(ReceiptStatusCode::DisallowedOpcode),
            21 => Ok(ReceiptStatusCode::CannotCompile),
            22 => Ok(ReceiptStatusCode::NoExportedContractMethod),
            23 => Ok(ReceiptStatusCode::OtherDeployError),

            30 => Ok(ReceiptStatusCode::ExecutionProperGasExhausted),
            31 => Ok(ReceiptStatusCode::RuntimeError),

            40 => Ok(ReceiptStatusCode::InternalExecutionProperGasExhaustion),
            41 => Ok(ReceiptStatusCode::InternalRuntimeError),
            42 => Ok(ReceiptStatusCode::InternalNotEnoughBalanceForTransfer),

            50 => Ok(ReceiptStatusCode::Else),

            _ => Err(()),
        }
    }
}

impl ReceiptStatusCode {
    pub fn is_success(&self) -> bool {
        ReceiptStatusCode::Success == *self
    }

    pub fn is_includable(&self) -> bool {
        ReceiptStatusCode::Success == *self 
        || ReceiptStatusCode::DisallowedOpcode == *self 
        || ReceiptStatusCode::CannotCompile == *self 
        || ReceiptStatusCode::NoExportedContractMethod == *self 
        || ReceiptStatusCode::OtherDeployError == *self
        || ReceiptStatusCode::ExecutionProperGasExhausted == *self
        || ReceiptStatusCode::RuntimeError == *self
        || ReceiptStatusCode::InternalExecutionProperGasExhaustion == *self
        || ReceiptStatusCode::InternalRuntimeError == *self
        || ReceiptStatusCode::InternalNotEnoughBalanceForTransfer == *self
    }

    pub fn is_retryable(&self) -> bool {
        ReceiptStatusCode::WrongNonce == *self 
        || ReceiptStatusCode::NotEnoughBalanceForGasLimit == *self 
        || ReceiptStatusCode::NotEnoughBalanceForTransfer == *self 
    }
}

impl Serializable<ReceiptStatusCode> for ReceiptStatusCode {}
impl Deserializable<ReceiptStatusCode> for ReceiptStatusCode {}