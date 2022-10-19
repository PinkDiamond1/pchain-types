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

use ed25519_dalek::{PublicKey, Signature, Verifier};
use sha2::{Sha256, Digest};
use crate::{crypto, receipt_status_codes, Serializable, Deserializable};

/// Transactions are authenticated, non-repudiable messages produced by external accounts 
/// to authorize blockchain state transitions, either through token transfer or smart contract
/// execution.
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct Transaction {
    /// Sender address in this transaction
    pub from_address: crypto::PublicAddress,
    /// Receiver address in this transaction
    pub to_address: crypto::PublicAddress,
    /// Value for transfer from sender to receiver
    pub value: u64,
    /// Tip for transfer from sender to validator
    pub tip: u64,
    /// Limit on gas for processing this transaction
    pub gas_limit: u64,
    /// The value used for balance deduction for gas used
    pub gas_price: u64,
    /// Transaction data
    pub data: Vec<u8>,
    /// Nonce. Accumulated number of transactions made by “From address”
    pub n_txs_on_chain_from_address: u64,
    /// Hash computed by hashing "Signature" of this transaction
    pub hash: crypto::Sha256Hash,
    /// An Ed25519 Signature on this transaction
    pub signature: crypto::Signature,
}

impl Transaction {
    pub fn verify_cryptographic_correctness(&self) -> Result<(), CryptographicallyIncorrectTransactionError> {
        // Verify the signature using the from_address (public key).
        let signed_msg = {
            let intermediate_txn = Transaction {
                from_address: self.from_address.to_owned(),
                to_address: self.to_address.to_owned(),
                value: self.value,
                tip: self.tip,
                gas_limit: self.gas_limit,
                gas_price: self.gas_price,
                data: self.data.to_owned(),
                n_txs_on_chain_from_address: self.n_txs_on_chain_from_address,
                hash: [0; 32],
                signature: [0; 64],
            };

            Transaction::serialize(&intermediate_txn)
        };
        let public_key = PublicKey::from_bytes(&self.from_address)
            .map_err(|_| CryptographicallyIncorrectTransactionError::InvalidFromAddress)?;
        let signature = Signature::from_bytes(&self.signature)
            .map_err(|_| CryptographicallyIncorrectTransactionError::InvalidSignature)?;
        let _ = public_key.verify(&signed_msg, &signature).map_err(|_| CryptographicallyIncorrectTransactionError::WrongSignature)?;

        // Verify the hash over the signature.
        let mut hasher = Sha256::new();
        hasher.update(&signature);
        if self.hash != Into::<crate::Sha256Hash>::into(hasher.finalize()) {
            Err(CryptographicallyIncorrectTransactionError::WrongHash)
        } else {
            Ok(())
        }

    }
}

pub enum CryptographicallyIncorrectTransactionError {
    InvalidFromAddress,
    InvalidSignature,
    WrongSignature,
    WrongHash,
}

/// Information that is required in transaction of contract
/// deployment. It is serialized into the field "data" of [Transaction]. 
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct DeployTransactionData {
    /// Contract wasm bytecode
    pub contract_code: Vec<u8>,
    /// Arguments to "init" method on the deploying contract. Equivalent to field "arguments" in [crate::CallData]
    pub contract_init_arguments: Vec<u8>
}

/// Events are messages produced by smart contract executions that are persisted on the blockchain
/// in a cryptographically-provable way. Events produced by transactions that call smart contracts
/// are stored in the `events` field of a Block in the order in which they are emitted.
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct Event { 
    /// Key of this event. It is created from contract execution
    pub topic: Vec<u8>,
    /// Value of this event. It is created from contract execution
    pub value: Vec<u8>,
}

/// Receipt defines the result of transaction execution.
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct Receipt {
    /// Receipt Status code
    pub status_code: receipt_status_codes::ReceiptStatusCode,
    /// Gas consumed for transaction execution
    pub gas_consumed: u64,
    /// Return value from transaction execution
    pub return_value: Vec<u8>,
    /// Vector of Event
    pub events: Vec<Event>,
}

impl Receipt {
    pub fn is_success(&self) -> bool {
        self.status_code.is_success()
    }

    pub fn is_includable(&self) -> bool {
        self.status_code.is_includable()
    }

    pub fn is_retryable(&self) -> bool {
        self.status_code.is_retryable()
    }
}

impl Serializable<Transaction> for Transaction {}
impl Deserializable<Transaction> for Transaction {}
impl Serializable<DeployTransactionData> for DeployTransactionData {}
impl Deserializable<DeployTransactionData> for DeployTransactionData {}
impl Serializable<Event> for Event {}
impl Deserializable<Event> for Event {}
impl Serializable<Receipt> for Receipt {}
impl Deserializable<Receipt> for Receipt {}