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

use std::convert::{TryFrom, TryInto};
use crate::{crypto, Transaction, Receipt, Serializable, Deserializable};

pub const BLOCK_GAS_LIMIT: usize = 67_500_000;
pub const BLOCK_SIZE_LIMIT: usize = 1_048_576;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub struct Block {
    pub header : BlockHeader,
    pub transactions : Vec<Transaction>,
    pub receipts : Vec<Receipt>,
}

/// Block header defines meta information of a block, including evidence for verifying validity of the block.
#[derive(Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct BlockHeader {
    /// Id of the blockchain. See [hotstuff_rs_types::messages::AppID]
    pub app_id: hotstuff_rs_types::messages::AppID,
    /// Block hash of this block
    pub hash: crypto::Sha256Hash,
    /// Identifier for a height of a block on the blockchain encoded as a number. Block number starts with 0. 
    /// For any other case, it is incremented by 1 over the block number of the previous block
    pub height: u64, 
    /// A cryptographic certificate which links a Block with its direct ancestor. See [hotstuff_rs_types::messages::QuorumCertificate]
    pub justify: hotstuff_rs_types::messages::QuorumCertificate,
    /// A cryptographic hash over the Block's Data. See [hotstuff_rs_types::messages::DataHash]
    pub data_hash: hotstuff_rs_types::messages::DataHash,
    /// Identifier for the set of block validation rules for the blockchain
    pub version_number :u64,
    /// Unix timestamp
    pub timestamp: u32,
    /// Merkle Tree root hash of transactions
    pub txs_hash : crypto::Sha256Hash,
    /// Merkle Tree root hash of current world-state
    pub state_hash : crypto::Sha256Hash,
    /// Merkle Tree root hash of receipts
    pub receipts_hash : crypto::Sha256Hash,
}

impl Serializable<Block> for Block {}
impl Deserializable<Block> for Block {}
impl Serializable<BlockHeader> for BlockHeader {}
impl Deserializable<BlockHeader> for BlockHeader {}

// Slot indexes definitions for
// pchain_types::Block and hotstuff_rs::msg_types::Block interoperability
impl Block {
    pub const NUM_SLOTS: usize = 5;
    pub const VERSION_SLOT: usize = 0;
    pub const TIMESTAMP_SLOT: usize = 1;
    pub const TXS_HASH_SLOT: usize = 2;
    pub const STATE_HASH_SLOT: usize = 3;
    pub const RECEIPTS_HASH_SLOT: usize = 4;
}

impl TryFrom<hotstuff_rs_types::messages::Block> for Block {
    type Error = TryFromHotStuffBlockError;

    fn try_from(block: hotstuff_rs_types::messages::Block) -> Result<Self, Self::Error> {
        
        if block.data.len() < Block::NUM_SLOTS {
            return Err(TryFromHotStuffBlockError::WrongNumberOfSlots)
        }

        let app_id = block.app_id;
        let block_hash: crypto::Sha256Hash = block.hash;
        let height: u64 = block.height;
        let justify: hotstuff_rs_types::messages::QuorumCertificate = block.justify;
        let data_hash: hotstuff_rs_types::messages::DataHash = block.data_hash;
    
        let version_number: u64 =  u64::from_le_bytes(block.data[Block::VERSION_SLOT].as_slice().try_into().map_err(|_| TryFromHotStuffBlockError::WrongVersionNumberLength)?);
        let timestamp: u32 = u32::from_le_bytes(block.data[Block::TIMESTAMP_SLOT].as_slice().try_into().map_err(|_| TryFromHotStuffBlockError::WrongTimestampLength)?);
        let txs_hash: crypto::Sha256Hash = block.data[Block::TXS_HASH_SLOT].as_slice().try_into().map_err(|_| TryFromHotStuffBlockError::WrongTxsHashLength)?;
        let state_hash: crypto::Sha256Hash = block.data[Block::STATE_HASH_SLOT].as_slice().try_into().map_err(|_| TryFromHotStuffBlockError::WrongStateHashLength)?;
        let receipts_hash: crypto::Sha256Hash = block.data[Block::RECEIPTS_HASH_SLOT].as_slice().try_into().map_err(|_| TryFromHotStuffBlockError::WrongReceiptsHashLength)?;
        
        let header: BlockHeader = BlockHeader {
            app_id,
            hash: block_hash,
            height,
            justify,
            data_hash,
            version_number,
            timestamp,
            txs_hash,
            state_hash,
            receipts_hash
        };

        let (transactions, receipts) = {
            let (txns_bs, receipts_bs) = {
                let num_remaining_slots = block.data.len() - Block::NUM_SLOTS;
                if num_remaining_slots % 2 != 0 {
                    return Err(TryFromHotStuffBlockError::WrongNumberOfSlots)
                }
                (
                    &block.data[Block::NUM_SLOTS..Block::NUM_SLOTS+(num_remaining_slots/2)], 
                    &block.data[Block::NUM_SLOTS+(num_remaining_slots/2)..]
                )
            }; 

            let mut transactions: Vec<Transaction> = Vec::with_capacity(txns_bs.len());
            for txn_bs in txns_bs {
                let txn = Transaction::deserialize(txn_bs).map_err(|_| TryFromHotStuffBlockError::WronglySerializedTransaction)?;
                transactions.push(txn.try_into().map_err(|e| TryFromHotStuffBlockError::WronglyAuthenticatedTransaction)?)
            }

            let mut receipts = Vec::with_capacity(receipts_bs.len());
            for receipt_bs in receipts_bs {
                receipts.push(Receipt::deserialize(receipt_bs).map_err(|_| TryFromHotStuffBlockError::WrongReceipt)?)
            }

            (transactions, receipts)
        };
     
        Ok(Block { 
            header,
            transactions,
            receipts
        })

    }
}

#[derive(Debug)]
pub enum TryFromHotStuffBlockError {
    WrongNumberOfSlots,
    WrongVersionNumberLength,
    WrongTimestampLength,
    WrongTxsHashLength,
    WrongStateHashLength,
    WrongReceiptsHashLength,
    WronglySerializedTransaction,
    WronglyAuthenticatedTransaction,
    WrongReceipt,
}
