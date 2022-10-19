# Types Subprotocol

ParallelChain F Protocol Types (pchain-types) defines data structures prescribed by the ParallelChain F Blockchain Protocol. These definitions help Web Applications, clients, and differing implementations of 'Node' software developed by different groups communicate with each other and exhibit correct, protocol-specified semantics and behavior. 

Table of Contents
- [Design Characteristics](#design-characteristics)
- [Specification of Encoding Formats](#specification-of-encoding-formats)
   - [Block Header](#block-header)
   - [Block Data](#block-data)
   - [Transaction](#transaction)
   - [Transaction Event](#transaction-event)
   - [Transaction Receipt](#transaction-receipt)
   - [Transaction Receipt Status Code](#transaction-receipt-status-code)
   - [Transaction CallData](#transaction-calldata)
   - [Merkle Proof](#merkle-proof)
   - [State Proof](#state-proof)
   - [Unpadded Base64URL](#unpadded-base64-url)

## Design Characteristics

ParallelChain F Protocol Types is designed with three desired properties in mind:
1. Obvious determinism. 
2. Compact encoding.
3. Fast

Determinism refers to the uniqueness of resulting bytes serialized from a data structure. Same data should be able to be serialized to one and only one series of bytes. Determinism is essential for Blockchain systems, since a single off-place byte totally changes cryptographic hashes and signatures produced over the data, making them much more difficult to verify. Additionally, this 'predictability' in the encoding allow serializer implementations to make more assumptions about the data, and possibly use memory allocation or parallel-processing tricks to speed up serialization and deserialization. 

Message format (e.g. definition of a block) is well-known, consistent and unchanged all over the time. 

Given the above design considerations, we achieve the characteristics by adopting [Borsh](https://borsh.io/) as underlying implementation.

A few features shared by all data types defined by ParallelChain F Protocol Types are: 
1. Data type consists of only unsigned integers and vector of bytes
2. Maximum size of the block should be less than 4GB (we use u32 as data type to define size, length, offset)
3. Little endian integer
4. Is interoperable with the Hotstuff-rs consensus crate. 

## Specification of Encoding Formats 

### Block Header

| Name | Type | Description |
|:--- |:--- |:--- |
|App id (ID)         |u64            |Id of the blockchain|
|Block hash (Hi)     |sha256 hash	 |Block hash of this block. = sha256 (ID, Hi, Bn, j, dh, Bv, t, Ht, Hs, Hr) where Hi are zeros as input.|
|Height (Bn)         |u64            |Identifier for a height of a block on the blockchain encoded as a number. Block number starts with 0. For any other case, it is incremented by 1 over the block number of the previous block| 
|Justify (j)             |QuorumCertificate | Quorum Certificate dervied from hotstuff_rs::msg_types|
|Data Hash (dh)          |sha256 hash    | Hash over msg_types::Data |
|Version number (Bv) |u64 |Identifier for the set of block validation rules for the blockchain|
|Timestamp (t)              |u32            |Unix timestamp. Number of seconds since 1970-01-01|
|Transaction Trie root hash (Ht)	|sha256 hash	|Merkle Tree root hash of transactions|
|State hash (Hs)            |sha256 hash	  |Merkle Tree root hash of current world-state|
|Receipt Trie root hash (Hr)|sha256 hash	|Merkle Tree root hash of receipts|

### Transaction

| Name | Type | Description |
|:--- |:--- |:--- |
|From address (fa)       |public address (32 bytes)      |Sender address in this transaction.<br />- an Ed25519 public key representing an external account|
|To address (ta)         |public address (32 bytes)	    |Receiver address in this transaction.<br/>- an Ed25519 public key representing an external account, or<br />- a contract address|
|Value (v)	                |u64	        |Value for transfer from sender to receiver|
|Tip (ti)	                |u64	        |Tip for transfer from sender to validator|
|Gas Limit (gl)	            |u64	        |Limit on gas for processing this transaction|
|Gas Price (gp)	            |u64	        |The value used for balance deduction for gas used|
|Data (td)	                |bytes	        |Transaction data|
|Num of Transaction (nt)	|u64    	    |Nonce. Accumulated number of transactions made by “From address”|
|Hash (h) 	                |sha256 hash    |Hash computed by hashing "Signature" of this transaction. = sha256 (tSig) |
|Signature (tsig)	        |signature (64 bytes)	    |An Ed25519 Signature on this transaction. = Sig(fa, ta, v, ti, gl, gp, nt, h, tsig, tds, td) where h and tsig are zeros as input|

### Transaction Receipt

| Name | Type | Description |
|:--- |:--- |:--- |
|Status code	    |u8	        |Receipt Status code|
|Gas consumed       |u64	    |Gas consumed for transaction execution|
|Return value       |bytes	    |Return value from transaction execution|
|Events             |bytes      |Vector of Event|

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

### Transaction Event
| Name | Type | Description |
|:--- |:--- |:--- |
|Topic	        |bytes	|Key of this event. It is created from contract execution|
|Value	        |bytes	|Value of this event. It is created from contract execution|

### Transaction CallData
Transaction CallData is used to select the Action or View method to be called in an EtoC transaction on a contract written using the ParallelChain F Smart Contract SDK and to provide the selected method with its arguments.

| Name | Type | Description |
|:---|:---|:---|
|Method name|String|Method to be called.|
|Arguments|Vec&lt;u8&gt;\*|The product of [Borsh](https://borsh.io/)-serializing a Vec&lt;Vec&lt;u8&gt;&gt; with `length == number of arguments*. Each inner Vec&lt;u8&gt; is a single argument, again, Borsh-serialized. i.e. Borsh([Borsh(Item 1), Borsh(Item 2), ...) |

### Merkle Proof
| Name | Type | Description |
|:--- |:--- |:--- |
|Root Hash|sha256 hash|Merkle root hash required in the proof|
|Total Leaves Count|u32|Number of Leaves in the Merkle Tree|
|Leaf Indices (li) |bytes|Vector of u32 integers. Integer li[i] represents the i-th leave to prove in the Trie|
|Leaf Hashes (lh) |bytes|Vector of sha256 hashes|
|Proof (prf)|bytes|Bytes used for verification|

__Note__: `MerkleProof` message definition is compatible to crate [rs_merkle](https://docs.rs/rs_merkle/latest/rs_merkle/). After deserializing to rust struct, fields can be passed to the function [verify](https://docs.rs/rs_merkle/latest/rs_merkle/struct.MerkleProof.html#method.verify).
```rust
// example 
let merkle_proof = MerkleProof::<Sha256>::try_from(proof)?;
let verify_result = merkle_proof.verify(root_hash, leaf_indices, leaf_hashes, total_leaves_count);
```

### State Proof

| Name | Type | Description |
|:--- |:--- |:--- |
|Root Hash|sha256 hash|Merkle root hash required in the proof|
|Items (pi) |bytes|Items are key-value pairs to verify with root hash and proof. Its data representation is Vec\<\(Vec\<u8\>, Option\<Vec\<u8\>\>\)\>|
|Proof (pp) |bytes|Proof is sequence of some nodes in trie traversed in pre-order traversal order. Its data representation is Vec\<Vec\<u8\>\>|

__Note__: `StateProofs` message definition is compatible to crate [trie-db](https://docs.rs/trie-db/latest/trie_db/).
```rust
// example 
verify_proof::<NoExtensionLayout, _, _, _>(&root_hash, &proof, items.iter())
```
### Unpadded Base64URL

In some components of the ParallelChain F system, There is a need for a way to represent arbitrary byte sequences as UTF-8, url-safe strings. Two examples are: 1. The `GET /account` routes, where clients have to specify address of the account to GET from by including its address in the request URL, and 2. `very-light`, which stores keypairs in a JSON file (`keypair.json`).

The standard bytes-to-characters encoding that all ParallelChain F components use is Base64URL, as defined in IETF RFC 4648, *without* padding characters. `protocol-types` exports an alias of String, `Base64URL`, alongside two functions `encode` and `decode` to convert from bytes to Base64URL and the other way around.
