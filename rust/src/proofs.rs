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

/// MerfleProof defines fields required in proving leaves hashes given a root hash and other related information
/// The fields are compatible to function `verify` used in crate [rs_merkle](https://docs.rs/rs_merkle/latest/rs_merkle/).
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct MerkleProof {
    /// Merkle root hash required in the proof
    pub root_hash: crypto::Sha256Hash,
    /// Number of Leaves in the Merkle Tree
    pub total_leaves_count: usize,
    /// Vector of u32 integers. Integer li[i] represents the i-th leave to prove in the Trie
    pub leaf_indices: Vec<usize>,
    /// Vector of sha256 hashes
    pub leaf_hashes: Vec<crypto::Sha256Hash>,
    /// Bytes used for verification
    pub proof: Vec<u8>,
}

/// StateProof is sequence of subset of nodes in trie traversed in pre-order traversal order.
pub type StateProof = Vec<Vec<u8>>;
/// StateProofItem contains key-value pair to verify with StateProof
pub type StateProofItem = (Vec<u8>, Option<Vec<u8>>);

/// StateProofs is compatible to functions in crate [trie-db](https://docs.rs/trie-db/latest/trie_db/)
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct StateProofs {
    /// Merkle root hash required in the proof
    pub root_hash :crypto::Sha256Hash,
    /// Items are key-value pairs to verify with root hash and proof. 
    pub items : Vec<StateProofItem>,
    /// Proof is sequence of some nodes in trie traversed in pre-order traversal order
    pub proof : StateProof
}

impl Serializable<MerkleProof> for MerkleProof {}
impl Deserializable<MerkleProof> for MerkleProof {}
impl Serializable<StateProofs> for StateProofs {}
impl Deserializable<StateProofs> for StateProofs {}
