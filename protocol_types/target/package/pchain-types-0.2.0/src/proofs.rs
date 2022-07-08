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

use std::mem;
use crate::encodings::{fmt_merkleproof, serialize, deserialize_u32, deserialize_32bytes};
use crate::{crypto, Serializable, Deserializable, Error, ErrorKind};

/// MerfleProof defines fields required in proving leaves hashes given a root hash and other related information
/// The fields are compatible to function `verify` used in crate [rs_merkle](https://docs.rs/rs_merkle/latest/rs_merkle/).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub root_hash :crypto::Sha256Hash,
    pub total_leaves_count: usize,
    pub leaf_indices :Vec<usize>,
    pub leaf_hashes :Vec<crypto::Sha256Hash>,
    pub proof :Vec<u8>,
}

impl Serializable for MerkleProof {
    fn serialize(msg: &MerkleProof) -> Vec<u8> {
        let leaf_indices_size = msg.leaf_indices.len() * mem::size_of::<u32>();
        let leaf_hashes_size = msg.leaf_hashes.len() * 32; // 32 bytes Sha256Hash
        // declare space to contains the bytes
        let mut ret :Vec<u8> = vec![0u8; fmt_merkleproof::BASESIZE + leaf_indices_size + leaf_hashes_size + msg.proof.len()];
        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_merkleproof::ROOTHASH       | msg.root_hash.as_slice(),
            fmt_merkleproof::TOTALLEAVES    | (msg.total_leaves_count as u32).to_le_bytes().as_slice(),
            fmt_merkleproof::LEAFINDSIZE    | (leaf_indices_size as u32).to_le_bytes().as_slice(),
            fmt_merkleproof::LEAFHASHSIZE   | (leaf_hashes_size as u32).to_le_bytes().as_slice(),
            fmt_merkleproof::PROOFSIZE      | (msg.proof.len() as u32).to_le_bytes().as_slice()
            => ret
        );
        // serialize the variable sized data
        let leaf_indices :Vec<u8> = msg.leaf_indices.iter().flat_map(|leaf_index|{
            (*leaf_index as u32).to_le_bytes()
        }).collect();
        let leaf_hashes :Vec<u8> = msg.leaf_hashes.iter().flat_map(|leaf_hash|{
            leaf_hash.clone()
        }).collect();

        let mut pos = fmt_merkleproof::BASESIZE;
        ret[pos..pos+leaf_indices_size].copy_from_slice(leaf_indices.as_slice());
        pos +=leaf_indices_size;
        ret[pos..pos+leaf_hashes_size].copy_from_slice(leaf_hashes.as_slice());
        pos += leaf_hashes_size;
        ret[pos..].copy_from_slice(msg.proof.as_slice());

        ret
    }
}

impl Deserializable<MerkleProof> for MerkleProof {
    fn deserialize(buf: &[u8]) -> Result<MerkleProof, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_merkleproof::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            root_hash,
            total_leaves_count,
            leaf_indices_size,
            leaf_hashes_size,
            proof_size
        ) = (
            deserialize_32bytes!(buf, fmt_merkleproof::ROOTHASH),
            deserialize_u32!(buf, fmt_merkleproof::TOTALLEAVES),
            deserialize_u32!(buf, fmt_merkleproof::LEAFINDSIZE),
            deserialize_u32!(buf, fmt_merkleproof::LEAFHASHSIZE),
            deserialize_u32!(buf, fmt_merkleproof::PROOFSIZE)
        );
        let total_leaves_count = total_leaves_count as usize;

        //deserialize variable sized data
        if buf.len() < fmt_merkleproof::BASESIZE + leaf_indices_size as usize + leaf_hashes_size as usize + proof_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let buf = &buf[fmt_merkleproof::BASESIZE..];
        let (leaf_indices_buf, remaining_buf) = buf.split_at(leaf_indices_size as usize);
        let (leaf_hashes_buf, proof_buf) = remaining_buf.split_at(leaf_hashes_size as usize);

        let mut leaf_indices = vec![];
        let mut pos = 0;
        while pos < leaf_indices_buf.len() {
            if leaf_indices_buf.len() < pos + mem::size_of::<u32>() {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            leaf_indices.push({
                let mut bs = [0u8; mem::size_of::<u32>()];
                bs.copy_from_slice(&leaf_indices_buf[pos..pos+mem::size_of::<u32>()]);
                u32::from_le_bytes(bs)
            } as usize);
            pos += mem::size_of::<u32>();
        }

        let mut leaf_hashes = vec![];
        let mut pos = 0;
        while pos < leaf_hashes_buf.len() {
            if leaf_hashes_buf.len() < pos + 32usize {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            leaf_hashes.push(deserialize_32bytes!(leaf_hashes_buf, (32, pos)));
            pos += 32;
        }
        let proof = proof_buf.to_vec();

        Ok(MerkleProof{
            root_hash,
            total_leaves_count,
            leaf_indices,
            leaf_hashes,
            proof
        })
    }
}

/// StateProof is sequence of subset of nodes in trie traversed in pre-order traversal order.
pub type StateProof = Vec<Vec<u8>>;
/// StateProofItem contains key-value pair to verify with StateProof
pub type StateProofItem = (Vec<u8>, Option<Vec<u8>>);

/// StateProofs is compatible to functions in crate [trie-db](https://docs.rs/trie-db/latest/trie_db/)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateProofs {
    pub root_hash :crypto::Sha256Hash,
    pub items : Vec<StateProofItem>,
    pub proof : StateProof
}

impl Serializable for StateProofs {
    fn serialize(msg: &StateProofs) -> Vec<u8> {
        let items = Vec::<StateProofItem>::serialize(&msg.items);
        let proof = StateProof::serialize(&msg.proof);
        [
            msg.root_hash.to_vec(),
            (items.len() as u32).to_le_bytes().to_vec(),
            (proof.len() as u32).to_le_bytes().to_vec(),
            items,
            proof
        ].concat()
    }
}

impl Deserializable<StateProofs> for StateProofs {
    fn deserialize(buf: &[u8]) -> Result<StateProofs, Error> {

        if buf.len() < 32 + 2*mem::size_of::<u32>() {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let root_hash = deserialize_32bytes!(buf, (32, 0));
        let buf = &buf[32..];

        let size_1 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];
        let size_2 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];

        if buf.len() < size_1 + size_2{
            return Err(Error::new(ErrorKind::IncorrectLength));
        }

        let (serialized_1, serialized_2) = buf.split_at(size_1);

        let items = Vec::<StateProofItem>::deserialize(serialized_1)?;
        let proof = StateProof::deserialize(serialized_2)?;

        Ok(StateProofs{
            root_hash,
            items,
            proof
        })
    }
}