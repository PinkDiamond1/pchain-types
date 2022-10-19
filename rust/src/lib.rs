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

//! run `cargo doc --open` to view rich documentation on the available types.

/// sc_params defines protocol types used to pass data into smart contract linear memory, including
/// [ParamsFromTransaction], [ParamsFromBlockchain], [CallData]
pub mod sc_params;

/// transaction defines transaction-related protocol types, including [Transaction], [Event], and [Receipt].
pub mod transaction; 

/// base64url defines a type which implements the basic operations on base64url (as defined in IETF RFC 4648) encoded binary data. base64url
/// is the *only* binary-to-text encoding scheme used in ParallelChain F. 
pub mod base64url;

/// generic types implementation of traits Serializable and Deserializable
pub mod blanket_impls;

/// block defines block-related protocol types, including [BlockHeader] and [Block].
pub mod block;

/// proof defines structures used for verifying a Merkle tree, including [MerkleProof], [StateProofs]
pub mod proofs;

/// crypto defines cryptography-related protocol types, including public addresses, secret keys, signatures, and hashes.
pub mod crypto;

/// receipt_status_codes defines ReceiptStatusCodes, a byte included in every Transaction Receipt that provides
/// a succinct way to describe what happened during the execution of the transaction. 
pub mod receipt_status_codes;


// Re-exports
pub use sc_params::*;
pub use blanket_impls::*;
pub use crypto::*;
pub use transaction::*;
pub use base64url::*;
pub use block::*;
pub use proofs::*;
pub use receipt_status_codes::*;


/// Serializable encapsulates implementation of serialization on data structures that are defined in pchain-types.
pub trait Serializable<T: borsh::BorshSerialize> {
    fn serialize(args: &T) -> Vec<u8> {
        args.try_to_vec().unwrap()
    }
}

/// Deserializable encapsulates implementation of deserialization on data structures that are defined in pchain-types.
pub trait Deserializable<T : borsh::BorshDeserialize> {
    fn deserialize(args: &[u8]) -> Result<T, std::io::Error> {
        T::try_from_slice(&args)
    }
}


#[cfg(test)]
mod test {

    use std::convert::TryFrom;

    use hotstuff_rs_types::messages;

    use crate::{
        Block, BlockHeader, Transaction, Receipt, Event,
        Serializable, Deserializable, DeployTransactionData, MerkleProof, StateProofs, ReceiptStatusCode,
    };

    use crate::{
        ParamsFromTransaction, ParamsFromBlockchain, CallData
    };

    macro_rules! measure_time {
        ($name:expr, $s:stmt) => {
            {
                let t_before = std::time::Instant::now();
                let ret = {$s};
                let t_after = std::time::Instant::now();
                let dur = t_after.duration_since(t_before);
                println!("{} time {:?}", stringify!($name), dur);
                ret
            }
        };
    }

    #[test]
    fn test_paramsfromtransaction() {
        let tx_param = ParamsFromTransaction {
            from_address: [0u8; 32],
            to_address: [1u8; 32],
            value: 99,
            data: vec![2u8; 101],
            transaction_hash: [3u8; 32]
        };
        let serialized = ParamsFromTransaction::serialize(&tx_param);

        let deserialized = ParamsFromTransaction::deserialize(&serialized.as_slice()).unwrap();

        assert_eq!(tx_param.from_address, deserialized.from_address);
        assert_eq!(tx_param.to_address, deserialized.to_address);
        assert_eq!(tx_param.value, deserialized.value);
        assert_eq!(tx_param.data, deserialized.data);
        assert_eq!(tx_param.transaction_hash, deserialized.transaction_hash);
    }

    #[test]
    fn test_paramsfromtransaction_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(ParamsFromTransaction::deserialize(&empty_serialized).is_err());

        // test by removing one byte with empty data
        let tx_param = ParamsFromTransaction {
            from_address: [0u8; 32],
            to_address: [1u8; 32],
            value: 99,
            data: vec![], // empty data
            transaction_hash: [3u8; 32]
        };
        let serialized = ParamsFromTransaction::serialize(&tx_param);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(ParamsFromTransaction::deserialize(&serialized).is_err());

        // test by removing one byte with data
        let tx_param = ParamsFromTransaction {
            from_address: [0u8; 32],
            to_address: [1u8; 32],
            value: 99,
            data: vec![2u8; 101],
            transaction_hash: [3u8; 32]
        };
        let serialized = ParamsFromTransaction::serialize(&tx_param);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(ParamsFromTransaction::deserialize(&serialized).is_err());
    }

    #[test]
    fn test_paramsfromblockchain() {
        let bc_param = ParamsFromBlockchain {
            this_block_number: 123,
            prev_block_hash: [99u8; 32],
            timestamp: 111110,
            random_bytes: [255u8; 32]
        };

        let serialized = ParamsFromBlockchain::serialize(&bc_param);

        let deserialized = ParamsFromBlockchain::deserialize(&serialized.as_slice()).unwrap();

        assert_eq!(bc_param.this_block_number, deserialized.this_block_number);
        assert_eq!(bc_param.prev_block_hash, deserialized.prev_block_hash);
        assert_eq!(bc_param.timestamp, deserialized.timestamp);
        assert_eq!(bc_param.random_bytes, deserialized.random_bytes);
    }

    #[test]
    fn test_paramsfromblockchain_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(ParamsFromBlockchain::deserialize(&empty_serialized).is_err());

        // test by removing one byte
        let bc_param = ParamsFromBlockchain {
            this_block_number: 123,
            prev_block_hash: [99u8; 32],
            timestamp: 111110,
            random_bytes: [255u8; 32]
        };
        let serialized = ParamsFromBlockchain::serialize(&bc_param);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(ParamsFromBlockchain::deserialize(&serialized).is_err());
    }

    #[test]
    fn test_calldata() {
        let call_data = CallData {
            method_name: "call data".to_string(),
            arguments: random_bytes::<34>().to_vec()
        };

        let serialized = CallData::serialize(&call_data);

        let deserialized = CallData::deserialize(&serialized.as_slice()).unwrap();

        assert_eq!(call_data.method_name, deserialized.method_name);
        assert_eq!(call_data.arguments, deserialized.arguments);
    }

    #[test]
    fn test_calldata_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(CallData::deserialize(&empty_serialized).is_err());
        
        // test by removing one byte
        let call_data = CallData {
            method_name: "call data".to_string(),
            arguments: random_bytes::<34>().to_vec()
        };
        let serialized = CallData::serialize(&call_data);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(CallData::deserialize(&serialized).is_err());

        // test for invalid utf8 bytes
        let mut serialized = CallData::serialize(&call_data);
        serialized[9] = 129;
        assert!(CallData::deserialize(&serialized).is_err());
    }

    #[test]
    fn test_block() {
        let block = Block{
            header: random_blockheader(),
            transactions: random_transactions(1000,1000,0, 1024),
            receipts: random_receipts(10, 10, 500,500,0, 1024),
        };

        let serialized = measure_time!(
            serialization,
            Block::serialize(&block)
        );
        println!("serialized size: {}", serialized.len());

        let deserialized = measure_time!(
            deserialization,
            Block::deserialize(&serialized).unwrap()
        );

        assert_block(&block, &deserialized)
    }

    #[test]
    fn test_block_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(Block::deserialize(&empty_serialized).is_err());

        let block = Block{
            header: random_blockheader(),
            transactions: random_transactions(1,1,128, 128),
            receipts: random_receipts(1, 1, 1,1,128, 128),
        };

        let serialized = Block::serialize(&block);
        let serialized_missing_last_byte = serialized[..serialized.len()-1].to_vec();
        assert!(Block::deserialize(&serialized_missing_last_byte).is_err());
    }

    #[test]
    fn test_block_should_be_deterministic() {
        let header_1 = random_blockheader();
        let header_2 = header_1.clone();

        assert_eq!(BlockHeader::serialize(&header_1), BlockHeader::serialize(&header_2));
        
        let transactions_1 = random_transactions(1000,1000,0, 1024);
        let transactions_2 = transactions_1.clone();
        
        assert_eq!(transactions_1, transactions_2);
        assert_eq!(Vec::<Transaction>::serialize(&transactions_1), Vec::<Transaction>::serialize(&transactions_2));

        let receipts_1 = random_receipts(10, 10, 500,500,0, 1024);
        let receipts_2 = receipts_1.clone();

        assert_eq!(receipts_1, receipts_2);
        assert_eq!(Vec::<Receipt>::serialize(&receipts_1), Vec::<Receipt>::serialize(&receipts_2));

        let block_1 = Block {
            header: header_1,
            transactions: transactions_1,
            receipts: receipts_1,
        };
        let block_2 = Block {
            header: header_2,
            transactions: transactions_2,
            receipts: receipts_2,
        };

        assert_eq!(Block::serialize(&block_1), Block::serialize(&block_2));
    }

    #[test]
    fn test_vec_blocks(){
        let mut blocks = vec![];
        for _ in 0..10 {
            blocks.push(Block{
                header: random_blockheader(),
                transactions: random_transactions(100,100,0, 1024),
                receipts: random_receipts(100,100,10,10,0, 1024),
            });
        }
        let serialized = measure_time!(
            serialization,
            Vec::<Block>::serialize(&blocks)
        );

        let deserialized = measure_time!(
            deserialization,
            Vec::<Block>::deserialize(&serialized).unwrap()
        );

        assert_eq!(blocks.len(), deserialized.len());

        for (i, block) in blocks.iter().enumerate() {
            assert_block(block, &deserialized[i]);
        }
    }

    #[test]
    fn test_blockheader(){
        let b = BlockHeader {
            app_id :1,
            version_number : 2,
            height: 1,
            timestamp : 3,
            justify : hotstuff_rs_types::messages::QuorumCertificate{
                view_number: 1,
                block_hash: [2u8; 32],
                sigs: hotstuff_rs_types::messages::SignatureSet {
                    signatures: vec![],
                    count_some: 0,
                },
            },
            hash : [2u8; 32],
            data_hash : [2u8; 32],
            txs_hash : [3u8; 32],
            state_hash : [4u8; 32],
            receipts_hash : [6u8; 32],
        };
        let serialized = BlockHeader::serialize(&b);

        let deserialized = BlockHeader::deserialize(&serialized.as_slice()).unwrap();

        assert_eq!(b.app_id, deserialized.app_id);
        assert_eq!(b.version_number, deserialized.version_number);
        assert_eq!(b.height, deserialized.height);
        assert_eq!(b.timestamp, deserialized.timestamp);
        assert_eq!(b.hash, deserialized.hash);
        assert_eq!(b.txs_hash, deserialized.txs_hash);
        assert_eq!(b.state_hash, deserialized.state_hash);
        assert_eq!(b.receipts_hash, deserialized.receipts_hash);
    }

    #[test]
    fn test_blockheader_error() {
        // test by removing one byte
        let b = BlockHeader {
            app_id :1,
            version_number : 2,
            height: 1,
            timestamp : 3,
            justify: hotstuff_rs_types::messages::QuorumCertificate{
                view_number: 1,
                block_hash: [2u8; 32],
                sigs: hotstuff_rs_types::messages::SignatureSet {
                    signatures: vec![],
                    count_some: 0,
                },
            },
            hash : [2u8; 32],
            data_hash : [2u8; 32],
            txs_hash : [3u8; 32],
            state_hash : [4u8; 32],
            receipts_hash : [6u8; 32],
        };
        let serialized = BlockHeader::serialize(&b);
        let serialized = serialized[..(serialized.len()-1)].to_vec();
        assert!(BlockHeader::deserialize(&serialized).is_err());
    }

    #[test]
    fn test_transaction() {
        // test by removing one byte
        let tx = Transaction{
            from_address: [0u8; 32],
            to_address: [1u8; 32],
            value: 1,
            tip: 2,
            gas_limit: 3,
            gas_price: 4,
            data: vec![2u8; 100],
            n_txs_on_chain_from_address: 5,
            hash: [3u8; 32],
            signature: [4u8; 64]
        };
        let serialized = Transaction::serialize(&tx);

        let deserialized = Transaction::deserialize(&serialized.as_slice()).unwrap();

        assert_transaction(&tx, &deserialized);
    }

    #[test]
    fn test_transaction_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(Transaction::deserialize(&empty_serialized).is_err());
       
        // test by removing one byte with empty data
        let tx = Transaction{
            from_address: [0u8; 32],
            to_address: [1u8; 32],
            value: 1,
            tip: 2,
            gas_limit: 3,
            gas_price: 4,
            data: vec![], // empty data
            n_txs_on_chain_from_address: 5,
            hash: [3u8; 32],
            signature: [4u8; 64]
        };
        let serialized = Transaction::serialize(&tx);
        let serialized = serialized[..(serialized.len()-1)].to_vec();
        assert!(Transaction::deserialize(&serialized).is_err());

        // test by removing one byte with data
        let tx = Transaction{
            from_address: [0u8; 32],
            to_address: [1u8; 32],
            value: 1,
            tip: 2,
            gas_limit: 3,
            gas_price: 4,
            data: vec![1u8; 100],
            n_txs_on_chain_from_address: 5,
            hash: [3u8; 32],
            signature: [4u8; 64]
        };
        let serialized = Transaction::serialize(&tx);
        let serialized = serialized[..(serialized.len()-1)].to_vec();
        assert!(Transaction::deserialize(&serialized).is_err());

    }

    #[test]
    fn test_vec_transactions(){
        let transactions = random_transactions(100,100,0, 1024);

        let serialized = Vec::<Transaction>::serialize(&transactions);

        let deserialized = Vec::<Transaction>::deserialize(&serialized).unwrap();

        assert_eq!(transactions.len(), deserialized.len());

        for (i, tx) in transactions.iter().enumerate() {
            let deserialized_tx = &deserialized[i];
            assert_transaction(&tx, deserialized_tx);
        }
    }

    #[test]
    fn test_transactiondatacontractdeployment() {
        let txdata = DeployTransactionData {
            contract_code: random_bytes::<100_000>().to_vec(),
            contract_init_arguments: random_bytes::<10_24>().to_vec(),
        };
        let serialized = DeployTransactionData::serialize(&txdata);
        let deserialzied = DeployTransactionData::deserialize(&serialized).unwrap();

        assert_eq!(txdata.contract_code, deserialzied.contract_code);
        assert_eq!(txdata.contract_init_arguments, deserialzied.contract_init_arguments);
    }

    #[test]
    fn test_transactiondatacontractdeployment_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(DeployTransactionData::deserialize(&empty_serialized).is_err());

        // test by removing one byte
        let txdata = DeployTransactionData {
            contract_code: random_bytes::<100_000>().to_vec(),
            contract_init_arguments: random_bytes::<10_24>().to_vec(),
        };
        let serialized = DeployTransactionData::serialize(&txdata);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(DeployTransactionData::deserialize(&serialized).is_err());
    }

    #[test]
    fn test_event() {
        let e = Event {
            topic: vec![10,20,30,40,50,60],
            value: vec![6,2,3]
        };

        let serialized = Event::serialize(&e);

        let deserialized = Event::deserialize(&serialized.as_slice()).unwrap();

        assert_eq!(e.topic, deserialized.topic);
        assert_eq!(e.value, deserialized.value);
    }

    #[test]
    fn test_event_error(){
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(Event::deserialize(&empty_serialized).is_err());

        // test by removing one byte
        let e = Event {
            topic: vec![10,20,30,40,50,60],
            value: vec![6,2,3]
        };
        let serialized = Event::serialize(&e);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(Event::deserialize(&serialized).is_err());
    }
    
    #[test]
    fn test_receipt() {
        let r = Receipt{
            gas_consumed:102,
            status_code: ReceiptStatusCode::InternalRuntimeError,
            return_value: vec![],
            events: random_events(10,10,0, 1024),
        };

        let serialized = Receipt::serialize(&r);
        let deserialized = Receipt::deserialize(&serialized.as_slice()).unwrap();
        
        assert_eq!(r.status_code, deserialized.status_code);
        assert_eq!(r.return_value, deserialized.return_value);
        assert_eq!(r.events.len(), deserialized.events.len());
        for (i, evt) in r.events.iter().enumerate() {
            let deserialized_evt = &deserialized.events[i];
            assert_eq!(evt.topic, deserialized_evt.topic);
            assert_eq!(evt.value, deserialized_evt.value);
        }
    }

    #[test]
    fn test_receipt_error(){
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(Receipt::deserialize(&empty_serialized).is_err());

        // test by removing one byte
        let r = Receipt{
            gas_consumed:102,
            status_code: ReceiptStatusCode::InternalRuntimeError,
            return_value: vec![],
            events: random_events(10,10,0, 1024),
        };
        let serialized = Receipt::serialize(&r);
        let serialized_missing_last_byte = serialized[..serialized.len()-1].to_vec();
        assert!(Receipt::deserialize(&serialized_missing_last_byte).is_err());
    }

    #[test]
    fn test_merkleproof(){
        let p = MerkleProof{
            root_hash :random_bytes::<32>(),
            total_leaves_count: 123,
            leaf_indices :vec![0,4,100],
            leaf_hashes : vec![random_bytes::<32>(),random_bytes::<32>(),random_bytes::<32>()],
            proof :random_bytes::<128>().to_vec()
        };
        let serialized = MerkleProof::serialize(&p);
        let deserialized = MerkleProof::deserialize(&serialized).unwrap();
        
        assert_eq!(p.root_hash, deserialized.root_hash);
        assert_eq!(p.total_leaves_count, deserialized.total_leaves_count);
        assert_eq!(p.leaf_indices, deserialized.leaf_indices);
        assert_eq!(p.leaf_hashes.len(), deserialized.leaf_indices.len());
        for (i, h) in p.leaf_hashes.iter().enumerate() {
            assert_eq!(*h, deserialized.leaf_hashes[i]);
        }
        assert_eq!(p.proof, deserialized.proof);
    }

    #[test]
    fn test_merkleproof_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(MerkleProof::deserialize(&empty_serialized).is_err());
        
        // test by removing one byte
        let p = MerkleProof{
            root_hash :random_bytes::<32>(),
            total_leaves_count: 123,
            leaf_indices :vec![0,4,100],
            leaf_hashes : vec![random_bytes::<32>(),random_bytes::<32>(),random_bytes::<32>()],
            proof :random_bytes::<128>().to_vec(),
        };
        let serialized = MerkleProof::serialize(&p);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(MerkleProof::deserialize(&serialized).is_err());

    }

    #[test]
    fn test_stateproofs() {
        let spfs = StateProofs {
            root_hash : random_bytes::<32>(),
            items : vec![
                (random_bytes::<21>().to_vec(), Some(random_bytes::<32>().to_vec())), 
                (random_bytes::<23>().to_vec(), None), 
                (random_bytes::<24>().to_vec(), Some(random_bytes::<35>().to_vec())), 
            ],
            proof : vec![random_bytes::<56>().to_vec(), random_bytes::<57>().to_vec(), random_bytes::<58>().to_vec()]
        };

        let serialized = StateProofs::serialize(&spfs);
        let deserialzied = StateProofs::deserialize(&serialized).unwrap();
        assert_eq!(spfs, deserialzied);
    }

    #[test]
    fn test_stateproofs_error() {
        // test empty vector
        let empty_serialized :Vec<u8> = vec![];
        assert!(StateProofs::deserialize(&empty_serialized).is_err());

        // test by removing one byte
        let spfs = StateProofs {
            root_hash : random_bytes::<32>(),
            items : vec![
                (random_bytes::<21>().to_vec(), Some(random_bytes::<34>().to_vec())), 
                (random_bytes::<23>().to_vec(), None), 
                (random_bytes::<24>().to_vec(), Some(random_bytes::<35>().to_vec())), 
            ],
            proof : vec![random_bytes::<56>().to_vec(), random_bytes::<57>().to_vec(), random_bytes::<58>().to_vec()]
        };

        let serialized = StateProofs::serialize(&spfs);
        let serialized = serialized[..serialized.len()-1].to_vec();
        assert!(StateProofs::deserialize(&serialized).is_err());
    }

    #[test]
    fn test_generics(){
        // u32
        let the_u32 = 1234_u32;
        let serialized = u32::serialize(&the_u32);
        let deserialized = u32::deserialize(&serialized).unwrap();
        assert_eq!(the_u32, deserialized);

        // u64
        let the_u64 = 1234123412341234_u64;
        let serialized = u64::serialize(&the_u64);
        let deserialized = u64::deserialize(&serialized).unwrap();
        assert_eq!(the_u64, deserialized);

        // Vec<u8>
        let vs = vec![];
        let serialized = Vec::<u8>::serialize(&vs);
        let deserialized = Vec::<u8>::deserialize(&serialized).unwrap();
        assert_eq!(vs, deserialized);

        let vs = random_bytes::<1024>().to_vec();
        let serialized = Vec::<u8>::serialize(&vs);
        let deserialized = Vec::<u8>::deserialize(&serialized).unwrap();
        assert_eq!(vs, deserialized);

        // Vec<Vec<u8>>
        let vvs = vec![vec![], vec![]];
        let serialized = Vec::<Vec<u8>>::serialize(&vvs);
        let deserialized = Vec::<Vec<u8>>::deserialize(&serialized).unwrap();
        vvs.into_iter().enumerate().for_each(|(i, s)|{
            assert_eq!(s, deserialized[i]);
        });

        let vvs = vec![random_bytes::<1024>().to_vec(), random_bytes::<1024>().to_vec()];
        let serialized = Vec::<Vec<u8>>::serialize(&vvs);
        let deserialized = Vec::<Vec<u8>>::deserialize(&serialized).unwrap();
        vvs.into_iter().enumerate().for_each(|(i, s)|{
            assert_eq!(s, deserialized[i]);
        });

        // Option<Vec<u8>>
        let none_vs = None;
        let serialized = Option::<Vec<u8>>::serialize(&none_vs);
        let deserialized = Option::<Vec<u8>>::deserialize(&serialized).unwrap();
        assert_eq!(none_vs, deserialized);

        let some_vs = Some(random_bytes::<1024>().to_vec());
        let serialized = Option::<Vec<u8>>::serialize(&some_vs);
        let deserialized = Option::<Vec<u8>>::deserialize(&serialized).unwrap();
        assert_eq!(some_vs, deserialized);

        // (Vec<u8>, Option<Vec<u8>>)
        let vs_none :(Vec<u8>, Option<Vec<u8>>) = (random_bytes::<128>().to_vec(), None);
        let serialized = <(Vec::<u8>, Option::<Vec::<u8>>)>::serialize(&vs_none);
        let deserialized = <(Vec::<u8>, Option::<Vec::<u8>>)>::deserialize(&serialized).unwrap();
        assert_eq!(vs_none, deserialized);

        let vs_some :(Vec<u8>, Option<Vec<u8>>) = (random_bytes::<128>().to_vec(), Some(random_bytes::<256>().to_vec()));
        let serialized = <(Vec::<u8>, Option::<Vec::<u8>>)>::serialize(&vs_some);
        let deserialized = <(Vec::<u8>, Option::<Vec::<u8>>)>::deserialize(&serialized).unwrap();
        assert_eq!(vs_some, deserialized);

    }

    #[test]
    fn test_status_codes() {
        [
            ReceiptStatusCode::Success,
            ReceiptStatusCode::WrongNonce,
            ReceiptStatusCode::NotEnoughBalanceForGasLimit,
            ReceiptStatusCode::NotEnoughBalanceForTransfer,
            ReceiptStatusCode::PreExecutionGasExhausted,
            ReceiptStatusCode::DisallowedOpcode,
            ReceiptStatusCode::CannotCompile,
            ReceiptStatusCode::NoExportedContractMethod,
            ReceiptStatusCode::OtherDeployError,
            ReceiptStatusCode::ExecutionProperGasExhausted,
            ReceiptStatusCode::RuntimeError,
            ReceiptStatusCode::InternalExecutionProperGasExhaustion,
            ReceiptStatusCode::InternalRuntimeError,
            ReceiptStatusCode::InternalNotEnoughBalanceForTransfer,
            ReceiptStatusCode::Else,
        ].to_vec().iter().for_each(|c|{
            let code = c.clone();
            let byte: u8 = code.clone().into();
            let code_from_byte: ReceiptStatusCode = ReceiptStatusCode::try_from(byte).unwrap();
            assert_eq!(code, code_from_byte);
        });
    }

    fn assert_block(block: &Block, deserialized: &Block) {
        assert_eq!(block.header.app_id, deserialized.header.app_id);
        assert_eq!(block.header.version_number, deserialized.header.version_number);
        assert_eq!(block.header.timestamp, deserialized.header.timestamp);
        assert_eq!(block.header.justify.block_hash, deserialized.header.hash);
        assert_eq!(block.header.hash, deserialized.header.hash);
        assert_eq!(block.header.txs_hash, deserialized.header.txs_hash);
        assert_eq!(block.header.state_hash, deserialized.header.state_hash);
        assert_eq!(block.header.receipts_hash, deserialized.header.receipts_hash);
        
        assert_eq!(block.transactions.len(), deserialized.transactions.len());
        assert_eq!(block.receipts.len(), deserialized.receipts.len());

        for (i, tx) in block.transactions.iter().enumerate() {
            let deserialized_tx = &deserialized.transactions[i];
            assert_transaction(tx, deserialized_tx);
        }

        for (i, recp) in block.receipts.iter().enumerate() {
            let deserialized_recp = &deserialized.receipts[i];
            assert_eq!(recp.gas_consumed, deserialized_recp.gas_consumed);
            assert_eq!(recp.status_code, deserialized_recp.status_code);
            assert_eq!(recp.return_value, deserialized_recp.return_value);
        }
    }

    fn assert_transaction(transaction: &Transaction, deserialized: &Transaction){
        assert_eq!(transaction.from_address, deserialized.from_address);
        assert_eq!(transaction.to_address, deserialized.to_address);
        assert_eq!(transaction.value, deserialized.value);
        assert_eq!(transaction.tip, deserialized.tip);
        assert_eq!(transaction.gas_limit, deserialized.gas_limit);
        assert_eq!(transaction.gas_price, deserialized.gas_price);
        assert_eq!(transaction.data, deserialized.data);
        assert_eq!(transaction.n_txs_on_chain_from_address, deserialized.n_txs_on_chain_from_address);
        assert_eq!(transaction.hash, deserialized.hash);
        assert_eq!(transaction.signature, deserialized.signature);
    }

    fn random_bytes<const N: usize>() -> [u8; N] {
        let mut res = [0u8; N];
        for i in 0..N {
            res[i] = rand::random::<u8>();
        }
        res
    } 

    fn random_bytes_dyn(n: usize) -> Vec<u8> {
        let mut res = Vec::with_capacity(n);
        for _ in 0..n {
            res.push(rand::random::<u8>());
        }
        res
    } 

    fn random_blockheader() -> BlockHeader {
        BlockHeader{
            app_id : rand::random::<u64>(),
            version_number : 2,
            height: rand::random::<u64>(),
            timestamp : rand::random::<u32>(),
            justify: hotstuff_rs_types::messages::QuorumCertificate{
                view_number: 1,
                block_hash: [2u8; 32],
                sigs: hotstuff_rs_types::messages::SignatureSet {
                    signatures: vec![],
                    count_some: 0,
                },
            },
            hash : [2u8; 32],
            data_hash : [2u8; 32],
            txs_hash : [3u8; 32],
            state_hash : random_bytes::<32>(),
            receipts_hash : random_bytes::<32>(),
        }
    }

    fn random_transaction(min_data_size :usize, max_data_size :usize) -> Transaction {
        let data_size = {
            let rand_size = max_data_size - min_data_size;
            min_data_size + if rand_size > 0 {rand::random::<usize>() % rand_size } else {0}
        };
        Transaction { 
            from_address: random_bytes::<32>(), 
            to_address: random_bytes::<32>(), 
            value: rand::random::<u64>(), 
            tip: rand::random::<u64>(), 
            gas_limit: rand::random::<u64>(), 
            gas_price: rand::random::<u64>(), 
            data: random_bytes_dyn(data_size), 
            n_txs_on_chain_from_address: rand::random::<u64>(), 
            hash: random_bytes::<32>(), 
            signature: random_bytes::<64>() 
        }
    }

    fn random_event(min_data_size :usize, max_data_size :usize) -> Event {
        let topic_data_size = {
            let rand_size = max_data_size - min_data_size;
            min_data_size + if rand_size > 0 {rand::random::<usize>() % rand_size } else {0}
        };
        let value_data_size =  {
            let rand_size = max_data_size - min_data_size;
            min_data_size + if rand_size > 0 {rand::random::<usize>() % rand_size } else {0}
        };
        Event {
            topic: random_bytes_dyn(topic_data_size),
            value: random_bytes_dyn(value_data_size)
        }
    }

    fn random_receipt(min_no_of_evts :usize, max_no_of_evts :usize, min_data_size :usize, max_data_size :usize) -> Receipt {
        let value_data_size =  {
            let rand_size = max_data_size - min_data_size;
            min_data_size + if rand_size > 0 {rand::random::<usize>() % rand_size } else {0}
        };
        Receipt {
            gas_consumed : rand::random::<u64>(),
            status_code: ReceiptStatusCode::Else,
            return_value : random_bytes_dyn(value_data_size),
            events: random_events(min_no_of_evts, max_no_of_evts, min_data_size, max_data_size)
        }
    }

    fn random_transactions(min_no_of_txs: usize, max_no_of_txs :usize, min_data_size :usize, max_data_size :usize) -> Vec<Transaction> {
        let no_of_txs = {
            let rand_size = max_no_of_txs - min_no_of_txs;
            min_no_of_txs + if rand_size > 0 {rand::random::<usize>() % rand_size} else {0}
        };
        let mut ret = vec![];
        for _ in 0..no_of_txs {
            ret.push(random_transaction(min_data_size, max_data_size));
        }
        ret
    }

    fn random_events(min_no_of_evts: usize, max_no_of_evts :usize, min_data_size :usize, max_data_size :usize) -> Vec<Event> {
        let no_of_evts = {
            let rand_size = max_no_of_evts - min_no_of_evts;
            min_no_of_evts + if rand_size > 0 {rand::random::<usize>() % rand_size} else {0}
        };
        let mut ret = vec![];
        for _ in 0..no_of_evts {
            ret.push(random_event(min_data_size, max_data_size));
        }
        ret
    }

    fn random_receipts(min_no_of_recps: usize, max_no_of_recps :usize, min_no_of_evts_per_recp :usize, max_no_of_evts_per_recp :usize, min_data_size :usize, max_data_size :usize) -> Vec<Receipt> {
        let no_of_recps = {
            let rand_size = max_no_of_recps - min_no_of_recps;
            min_no_of_recps + if rand_size > 0 {rand::random::<usize>() % rand_size} else {0}
        };
        let mut ret = vec![];
        for _ in 0..no_of_recps {
            ret.push(
                random_receipt(min_no_of_evts_per_recp, max_no_of_evts_per_recp, min_data_size, max_data_size)
            );
        }
        ret
    }
}