# ParallelChain F Protocol Types

ParallelChain F Protocol Types (pchain_types) defines data structures and serialization formats prescribed by the ParallelChain F Blockchain Protocol. These definitions help Web Applications, light clients, and differing implementations of 'Node' software developed by different groups communicate with each other and exhibit correct, protocol-specified semantics and behavior. 

Table of Contents
- [Design Characteristics](#design-characteristics)
- [Specification of Encoding Formats](#specification-of-encoding-formats)
   - [Block Header](#block-header)
   - [Block Data](#block-data)
   - [Transaction](#transaction)
   - [Transaction Event](#transaction-event)
   - [Transaction Receipt](#transaction-receipt)
   - [Transaction Receipt Status Code](#transaction-receipt-status-code)
   - [Merkle Proof](#merkle-proof)
   - [State Proof](#state-proof)
   - [More general types](#generalizations)

## Design Characteristics

ParallelChain F Protocol Types is designed with three desired properties in mind:
1. Obvious determinism. 
2. Compact encoding.
3. Fast, parallelizable serialization and deserialization.

Determinism refers to the uniqueness of resulting bytes serialized from a data structure. Same data should be able to be serialized to one and only one series of bytes. Determinism is essential for Blockchain systems, since a single off-place byte totally changes cryptographic hashes and signatures produced over the data, making them much more difficult to verify. Additionally, this 'predictability' in the encoding allow serializer implementations to make more assumptions about the data, and possibly use memory allocation or parallel-processing tricks to speed up serialization and deserialization. 

General-purpose serialization protocols (e.g. Protocol Buffers) do not require, nor guarantee determinism. Therefore, we do not rely on third parity serialization protocol.

Message format (e.g. definition of a block) is well-known, consistent and unchanged all over the time. Hence, extra information to describe the message format itself is not required, allowing for more compact messages.

Given the above design considerations, we achieve the characteristics by enforcing the data to be strictly sized. In case of requiring variable length of data, we specify the length of the data beforehand, which is similar to the data pattern of [TLV](https://en.wikipedia.org/wiki/Type%E2%80%93length%E2%80%93value).

A few features shared by all data types defined by ParallelChain F Protocol Types are: 
1. Data type consists of only unsigned integers and vector of bytes
2. Maximum size of the block should be less than 4GB (we use u32 as data type to define size, length, offset)
3. Little endian integer

## Specification of Encoding Formats 

### Block Header

| Name | Type | Description |
|:--- |:--- |:--- |
|Blockchain id (Bi)         |u64            |Id of the blockchain|
|Block version number (Bv)  |u64	        |Block version number. Must be the same among all nodes in blockchain|
|Timestamp (t)              |u32            |Unix timestamp. Number of seconds since 1970-01-01|
|Previous block hash (Hp)	|sha256 hash	|Block hash of previous block|
|This block hash (Hi)       |sha256 hash	|Block hash of this block. = sha256 (Bi, Bv, t, Hp, Hi, Hs, Ht, Hr, P, Sig) where Hi are zeros as input.|
|State hash (Hs)            |sha256 hash	|Merkle Tree root hash of current world-state|
|Transaction Trie root hash (Ht)	|sha256 hash	|Merkle Tree root hash of transactions|
|Receipt Trie root hash (Hr)|sha256 hash	|Merkle Tree root hash of receipts|
|Proposer public key (P)    |public address (32 bytes)	|Public key of proposer which is either<br/>- an Ed25519 public key representing an external account, or<br />- a contract address |
|Signature on block header (Sig) |signature (64 bytes) |An Ed25519 Signature on the block header. = Signature (Bi, Bv, t, Hp, Hi, Hs, Ht, Hr, P, Sig) where Hi and Sig are zeros as input.|

### Block Data

| Name | Type | Description |
|:--- |:--- |:--- |
|Transaction block data size |u32   |Size of data section (Dt)|
|Receipt block data size     |u32   |Size of data section (Dr)|
|Transaction Data (Dt)	     |bytes |Bytes concatenation over all Transaction Entries|
|Receipt Data (Dr)           |bytes |Bytes concatenation over all Receipt Entries|

#### Transaction

| Name | Type | Description |
|:--- |:--- |:--- |
|From address (fa)       |public address (32 bytes)      |Sender address in this transaction.<br />- an Ed25519 public key representing an external account|
|To address (ta)         |public address (32 bytes)	    |Receiver address in this transaction.<br/>- an Ed25519 public key representing an external account, or<br />- a contract address|
|Value (v)	                |u64	        |Value for transfer from sender to receiver|
|Tip (ti)	                |u64	        |Tip for transfer from sender to validator|
|Gas Limit (gl)	            |u64	        |Limit on gas for processing this transaction|
|Gas Price (gp)	            |u64	        |The value used for balance deduction for gas used|
|Num of Transaction (nt)	|u64    	    |Nonce. Accumulated number of transactions made by “From address”|
|Hash (h) 	                |sha256 hash    |Hash computed by hashing "Signature" of this transaction entry. = sha256 (tSig) |
|Signature (tsig)	        |signature (64 bytes)	    |An Ed25519 Signature on this transaction entry. = Sig(fa, ta, v, ti, gl, gp, nt, h, tsig, tds, td) where h and tsig are zeros as input|
|Data size (tds)	        |u32	        |Data size (in bytes) of subsequent data|
|Data (td)	                |bytes	        |Transaction data|

#### Transaction Receipt

| Name | Type | Description |
|:--- |:--- |:--- |
|Gas consumed       |u64	    |Gas consumed for execution of i-th transaction in this block, where i is the index of this entry. (number of receipt entries must be equal to number of transaction entries)|
|Status code	    |u8	        |Receipt Status code|
|Return value size	|u32	    |Size of return value (in bytes)|
|Events size        |u32        |Size of events sub-entries (in bytes)|
|Return value       |bytes	    |Return value from transaction execution|
|Events             |bytes      |Bytes concatenation over all Event Sub-entries|

### Transaction Receipt Status Code

Categorization  Receipt Status Code
- 1x - Pre-Inclusion Decision failures. Not included in blocks.
- 2x - Deploy errors.
- 3x - EtoC errors (not in internal transaction)
- 4x - Internal transaction errors

|Code|Status|Description|
|:---|:---|:---|
|00|Success|Successful transaction|
|10| Wrong Nonce | Incorrect account nonce is used in transaction. Each nonce should only be used once and in sequential order.|
|11| Not Enough Balance For Gas Limit | Not enough balance to pay for gas limit.|
|12| Not Enough Balance For Transfer | Not enough balance to pay for transfer.|
|13| Pre-Execution Gas Exhausted | Gas limit was insufficient to cover pre-execution costs, or set the gas limit too low. |
|20| Disallowed Opcode | Fail to loal contract because the contract bytecode contains disallowed opcodes.|
|21| Cannot Compile | Contract cannot be compiled into machine code (it is probably invalid WASM).|
|22| No Exported ContractMethod | Contract does not export the METHOD_CONTRACT method. |
|23| Other Deploy Error | Deployment failed for some other reasons.|
|30| Execution Proper Gas Exhausted | Gas limit was insufficient to cover execution proper costs.|
|31| Runtime Error| Runtime error during execution proper of the entree smart contract.|
|40| Internal Execution Proper Gas Exhaustion | Gas limit was insufficient to cover execution proper costs of an internal transaction.|
|41| Internal Runtime Error | Runtime error during execution proper of an internal transaction.|
|42| Internal Not Enough Balance For Transfer | Not enough balance to pay for transfer in an internal transaction.|
|43| Other Error| Other error.|

#### Transaction Event
| Name | Type | Description |
|:--- |:--- |:--- |
|Topic size	    |u32	|Size of topic (in bytes)|
|Value size	    |u32	|Size of value (in bytes)|
|Topic	        |bytes	|Key of this event. It is created from contract execution|
|Value	        |bytes	|Value of this event. It is created from contract execution|

### Merkle Proof
| Name | Type | Description |
|:--- |:--- |:--- |
|Root Hash|sha256 hash|Merkle root hash required in the proof.|
|Total Leaves Count|u32|Number of Leaves in the Merkle Tree.|
|Leaf Indices size|u32|Size of Leaf Indices (li) in bytes.|
|Leaf Hashes size|u32|Size of Leaf Hashes (lh) in bytes.|
|Proof size|u32|Size of proof (prf) in bytes.|
|Leaf Indices (li) |bytes|Byte concatenation over u32 integers. Integer li[i] represents the i-th leave to prove in the Trie.|
|Leaf Hashes (lh) |bytes|Byte concatenation over sha256 hashes.|
|Proof (prf)|bytes|Bytes used for verification.|

__Note__: `MerkleProof` message definition is compatible to crate [rs_merkle](https://docs.rs/rs_merkle/latest/rs_merkle/). After deserializing to rust struct, fields can be passed to the function [verify](https://docs.rs/rs_merkle/latest/rs_merkle/struct.MerkleProof.html#method.verify).
```rust
// example 
let merkle_proof = MerkleProof::<Sha256>::try_from(proof)?;
let verify_result = merkle_proof.verify(root_hash, leaf_indices, leaf_hashes, total_leaves_count);
```

### State Proof

| Name | Type | Description |
|:--- |:--- |:--- |
|Root Hash|sha256 hash|Merkle root hash required in the proof.|
|Size of Items|u32|Size of Items (pi) in bytes|
|Size of Proof|u32|Size of Proof (pp) in bytes|
|Items (pi) |bytes|Items are key-value pairs to verify with root hash and proof. Its data representation is Vec\<\(Vec\<u8\>, Option\<Vec\<u8\>\>\)\>. Please see section [Generalization](#generalization)|
|Proof (pp) |bytes|Proof is sequence of some nodes in trie traversed in pre-order traversal order. Its data representation is Vec\<Vec\<u8\>\>. Please see section [Generalization](#generalization)|

__Note__: `StateProofs` message definition is compatible to crate [trie-db](https://docs.rs/trie-db/latest/trie_db/).
```rust
// example 
verify_proof::<NoExtensionLayout, _, _, _>(&root_hash, &proof, items.iter())
```

## Generalizations

Protocol Types supports serialization on data types in rust such as Vec, Option, and tuple, where the generic type T that used can be u8, Vec\<u8\> or the mentioned message definitions.

### Vector of T (Rust: Vec<T>)

| Name | Type |
|:-- |:--- |
| Number of Serialized data (=N)| u32|
| Size of Entry 0| u32 |
| Size of Entry 1| u32 |
|.. |.. |
| Size of Entry N-1| u32|
| Data for Entry 0| bytes|
| Data for Entry 1| bytes|
|.. |.. |
| Data for Entry N-1| bytes|

### Optional Field (Rust: Option<T>)

| Name | Type | Description |
|:--- |:--- |:--- |
|Indicator |u8|value is zero or one indicating whether it contains data section|
|Data section |bytes|Data that can be u8, Vec\<u8\> or any defined message |

### 2-Tuple

|Name|Type|
|:--- |:--- |
|Size of data section 1|u32|
|Size of data section 2|u32|
|Data section 1|bytes|
|Data section 2|bytes|