use std::mem;
use std::thread::{self, JoinHandle};
use crate::encodings::{
    fmt_sizes, 
    fmt_blockheader, 
    serialize,
    deserialize_u32, 
    deserialize_u64, 
    deserialize_32bytes,
    deserialize_64bytes};
use crate::{Transaction, Receipt, crypto, Serializable, Deserializable, Error, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub header : BlockHeader,
    pub transactions : Vec<Transaction>,
    pub receipts : Vec<Receipt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[deprecated="No necessary use cases. Please consider using Vec<Vec<u8> or Vec<Block> directly."]
pub struct Blocks {
    pub blocks : Vec<Block>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub blockchain_id : u64,
    pub block_version_number :u64,
    /// Unix timestamp
    pub timestamp :u32, 
    pub prev_block_hash : crypto::Sha256Hash,
    pub this_block_hash : crypto::Sha256Hash,
    pub txs_hash : crypto::Sha256Hash,
    pub state_hash : crypto::Sha256Hash,
    pub receipts_hash : crypto::Sha256Hash,
    pub proposer_public_key  : crypto::PublicAddress,
    pub signature : crypto::Signature
}

/// unsafe pointer to be shared with threads
struct Ptr<T>(*const T);
unsafe impl<T> Send for Ptr<T> {}

impl Serializable for Block {

    fn serialize<'a>(msg: &'a Block) -> Vec<u8> {
        let raw_blockheader = BlockHeader::serialize(&msg.header);

        // transactions serialization
        let tx_ptr = Ptr(&msg.transactions as *const Vec<Transaction>);
        let tx_join_handle : JoinHandle<Vec<u8>> = thread::spawn(move || {
            let transactions = unsafe {&*tx_ptr.0};
            // allocate all memory once. Avoid allocating memory in iterations because it slows down the process
            let total_size = transactions.iter().map(|t|{
                Transaction::size_of(t)
            }).sum();
            let mut return_slice = vec![0u8; total_size];
            // serialize sequentially
            let mut pos :usize = 0;
            transactions.iter().for_each(|t| {
                pos += Transaction::serialize_to_slice(t, &mut return_slice[pos..]);
            });
            return_slice
        });

        // receipts serialization
        let recp_ptr = Ptr(&msg.receipts as *const Vec<Receipt>);
        let recp_join_handle : JoinHandle<Vec<u8>> = thread::spawn(move || {
            let receipts = unsafe {&*recp_ptr.0};
            // allocate all memory once. Avoid allocating memory in iterations because it slows down the process
            let total_size :usize = receipts.iter().map(|r|{
                Receipt::size_of(r)
            }).sum();
            let mut return_slice = vec![0u8; total_size];
            // serialize sequentially
            let mut pos :usize = 0;
            receipts.iter().for_each(|r| {
                pos += Receipt::serialize_to_slice(r, &mut return_slice[pos..]);
            });
            return_slice
        });

        // join all serialization threads for in sequential order
        let tx_data = tx_join_handle.join().unwrap();
        let tx_size = (tx_data.len() as u32).to_le_bytes().to_vec();

        let recp_data = recp_join_handle.join().unwrap();
        let recp_size = (recp_data.len() as u32).to_le_bytes().to_vec();

        // return serialized Block
        [
            raw_blockheader, 
            [tx_size, recp_size].concat(),
            [tx_data, recp_data].concat()
        ].concat()
    }
}

impl Deserializable<Block> for Block {
    
    fn deserialize(buf: &[u8]) -> Result<Block, Error> {
        let blockheader = BlockHeader::deserialize(buf)?;

        let blockdata_offset = fmt_blockheader::BASESIZE as usize;

        // decode size of transactions and receipts
        let buf = &buf[blockdata_offset..];
        if buf.len() < fmt_sizes::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let tx_size = deserialize_u32!(buf, fmt_sizes::TRANSACTION) as usize;
        let recp_size = deserialize_u32!(buf, fmt_sizes::RECEIPT) as usize;

        // get slices for transactions and receipts
        let buf = &buf[fmt_sizes::BASESIZE..];
        if buf.len() < tx_size + recp_size {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (raw_transactions, raw_receipts) = buf.split_at(tx_size);

        // deserialize transactions slice
        let raw_transactions = raw_transactions.to_vec(); // convert to vec for passing the content to thread
        let tx_join_handle = thread::spawn(move ||{
            let mut transactions = vec![];
            let mut pos = 0usize;
            while pos < raw_transactions.len() {
                let raw_buf = &raw_transactions[pos..];
                let size_of_i_th_tx = Transaction::size_from_slice(raw_buf)?;
                transactions.push(
                    Transaction::deserialize(&raw_buf[..size_of_i_th_tx])?
                );
                pos += size_of_i_th_tx;
            }
            Ok(transactions)
        });

        // deserialize receipts slice
        let raw_receipts = raw_receipts.to_vec(); // convert to vec for passing the content to thread
        let recp_join_handle = thread::spawn(move || {
            let mut receipts = vec![];
            let mut pos = 0usize;
            while pos < raw_receipts.len() {
                let raw_buf = &raw_receipts[pos..];
                let size_of_i_th_recp = Receipt::size_from_slice(&raw_buf)?;
                receipts.push(
                    Receipt::deserialize(&raw_buf[..size_of_i_th_recp])?
                );
                pos += size_of_i_th_recp;
            }
            Ok(receipts)
        });

        let transactions = tx_join_handle.join().unwrap()?;
        let receipts = recp_join_handle.join().unwrap()?;

        Ok(Block{
            header: blockheader,
            transactions,
            receipts
        })
    }

}

#[allow(deprecated)]
impl Serializable for Blocks {

    fn serialize(msg: &Blocks) -> Vec<u8> {
        let blocks :Vec<u8> = msg.blocks.iter().flat_map(|block|{
            let serialized_block = Block::serialize(&block);
            let block_size = serialized_block.len();
            let mut entry_bytes = vec![0u8; mem::size_of::<u32>() + block_size];
            entry_bytes[0..mem::size_of::<u32>()].copy_from_slice((block_size as u32).to_le_bytes().as_slice());
            entry_bytes[mem::size_of::<u32>()..].copy_from_slice(serialized_block.as_slice());
            entry_bytes
        }).collect();
        blocks
    }

}

#[allow(deprecated)]
impl Deserializable<Blocks> for Blocks {

    fn deserialize(buf: &[u8]) -> Result<Blocks, Error> {
        let mut blocks :Vec<Block> = vec![];
        let mut pos = 0usize;
        while pos < buf.len() {
            // get the size of block
            if buf.len() < pos + mem::size_of::<u32>() {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            let block_size ={
                let mut bs = [0u8; mem::size_of::<u32>()];
                bs.copy_from_slice(& buf[pos..pos+mem::size_of::<u32>()]);
                u32::from_le_bytes(bs)
            };

            // get the block
            if buf.len() < pos + mem::size_of::<u32>() + block_size as usize {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            let block = Block::deserialize(&buf[pos+mem::size_of::<u32>()..pos+mem::size_of::<u32>()+block_size as usize])?;
            blocks.push(block);
            pos += mem::size_of::<u32>() + block_size as usize;
        }

        Ok(Blocks {
            blocks
        })
    }

}

impl Serializable for BlockHeader {
    fn serialize(msg: &BlockHeader) -> Vec<u8> {

        // declare space to contains the bytes
        let mut ret :Vec<u8> = vec![0u8; fmt_blockheader::BASESIZE];

        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_blockheader::ID          | msg.blockchain_id.to_le_bytes().as_slice(),
            fmt_blockheader::VERSION     | msg.block_version_number.to_le_bytes().as_slice(),
            fmt_blockheader::TIMESTAMP   | msg.timestamp.to_le_bytes().as_slice(),
            fmt_blockheader::PREVHASH    | msg.prev_block_hash.as_slice(),
            fmt_blockheader::BLOCKHASH   | msg.this_block_hash.as_slice(),
            fmt_blockheader::STATEHASH   | msg.state_hash.as_slice(),
            fmt_blockheader::TXSHASH     | msg.txs_hash.as_slice(),
            fmt_blockheader::RECEIPTHASH | msg.receipts_hash.as_slice(),
            fmt_blockheader::PUBKEY      | msg.proposer_public_key.as_slice(),
            fmt_blockheader::SIGNATURE   | msg.signature.as_slice()
            => ret
        );

        ret
    }

}

impl Deserializable<BlockHeader> for BlockHeader {
    fn deserialize(buf: &[u8]) -> Result<BlockHeader, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_blockheader::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength))
        }
        let (
            blockchain_id,
            block_version_number,
            timestamp,
            prev_block_hash,
            this_block_hash,
            state_hash,
            txs_hash,
            receipts_hash,
            proposer_public_key,
            signature
        ) = (
            deserialize_u64!(buf, fmt_blockheader::ID),
            deserialize_u64!(buf, fmt_blockheader::VERSION),
            deserialize_u32!(buf, fmt_blockheader::TIMESTAMP),
            deserialize_32bytes!(buf, fmt_blockheader::PREVHASH),
            deserialize_32bytes!(buf, fmt_blockheader::BLOCKHASH),
            deserialize_32bytes!(buf, fmt_blockheader::STATEHASH),
            deserialize_32bytes!(buf, fmt_blockheader::TXSHASH),
            deserialize_32bytes!(buf, fmt_blockheader::RECEIPTHASH),
            deserialize_32bytes!(buf, fmt_blockheader::PUBKEY),
            deserialize_64bytes!(buf, fmt_blockheader::SIGNATURE)
        );

        Ok(BlockHeader {
            blockchain_id,
            block_version_number,
            timestamp,
            prev_block_hash,
            this_block_hash,
            txs_hash,
            state_hash,
            receipts_hash,
            proposer_public_key,
            signature
        })
    }
}
