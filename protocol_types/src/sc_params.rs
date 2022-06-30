use std::mem;
use crate::encodings::{serialize, deserialize_u32, deserialize_u64, deserialize_32bytes};
use crate::{crypto, Serializable, Deserializable, Error, ErrorKind};


/// Data format of Parameter from Transaction
mod fmt_paramsfromtransaction {
    // define format for ParamsFromTransaction
    define_format!(
        // field        | size  | offset
        FROMADDRESS     | 32    | 0,
        TOADDRESS       | 32    | 32,
        VALUE           | 8     | 64,
        HASH            | 32    | 72,
        DATASIZE        | 4     | 104
        // DATA         | bytes with variable size
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of Parameter from Blockchain
mod fmt_paramsfromblockchain {
    // define format for ParamsFromBlockchain
    define_format!(
        // field    | size  | offset
        BLOCKNUM    | 8     | 0,
        PREVHASH    | 32    | 8,
        TIMESTAMP   | 4     | 40,
        RANDOMBYTES | 32    | 44
        => BASESIZE // sum of size of fields with fixed size
    );
}

mod fmt_calldata {
    define_format!(
        // field        | size  | offset
        METHOD_SIZE     | 4     | 0,
        ARGS_SIZE       | 4     | 4
        // METHODNAME   | bytes with variable size
        // ARGS         | bytes with variable size
        => BASESIZE // sum of size of fields with fixed size
    );
}

#[derive(Debug, Clone)]
pub struct ParamsFromTransaction {
    pub from_address :crypto::PublicAddress,
    pub to_address :crypto::PublicAddress,
    pub data :Vec<u8>,
    pub value :u64,
    pub transaction_hash :crypto::Sha256Hash
}

#[derive(Debug, Clone)]
pub struct ParamsFromBlockchain {
    pub this_block_number :u64,
    pub prev_block_hash :crypto::Sha256Hash,
    /// Unix timestamp
    pub timestamp :u32,
    pub random_bytes :crypto::Sha256Hash,
}

/// CallData defines the data format that passes to entry point of the contact
/// 
/// The struct contains primitive types which are serialized into the field "argument" in smart_contract::Transaction.
/// Before that, Transaction::argument is appended 4 bytes at the front to indicate the format version
pub struct CallData {
    /// function name of contract with entrypoint methods. Empty string indicates the contract without entrypoint methods
    pub method_name :String,

    /// arguments to function (entrypoint method)
    /// In contract with entrypoint methods, the arguments should be deserialized to vector of Vec<u8> and then pass as function arguments
    pub arguments :Vec<u8>
}

impl Serializable for ParamsFromTransaction {
    fn serialize(msg: &ParamsFromTransaction) -> Vec<u8> {
        // declare space to contains the bytes
        let mut ret :Vec<u8> = vec![0u8; fmt_paramsfromtransaction::BASESIZE + msg.data.len()];
        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_paramsfromtransaction::FROMADDRESS | msg.from_address.as_slice(),
            fmt_paramsfromtransaction::TOADDRESS   | msg.to_address.as_slice(),
            fmt_paramsfromtransaction::VALUE       | msg.value.to_le_bytes().as_slice(),
            fmt_paramsfromtransaction::HASH        | msg.transaction_hash.as_slice(),
            fmt_paramsfromtransaction::DATASIZE    | (msg.data.len() as u32).to_le_bytes().as_slice()
            => ret
        );
        // serialize the variable sized data
        ret[fmt_paramsfromtransaction::BASESIZE..].copy_from_slice(msg.data.as_slice());
        ret
    }
}

impl Deserializable<ParamsFromTransaction> for ParamsFromTransaction {
    fn deserialize(buf: &[u8]) -> Result<ParamsFromTransaction, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_paramsfromtransaction::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            from_address,
            to_address,
            value,
            transaction_hash,
            data_size
        ) = (
            deserialize_32bytes!(buf, fmt_paramsfromtransaction::FROMADDRESS),
            deserialize_32bytes!(buf, fmt_paramsfromtransaction::TOADDRESS),
            deserialize_u64!(buf, fmt_paramsfromtransaction::VALUE),
            deserialize_32bytes!(buf, fmt_paramsfromtransaction::HASH),
            deserialize_u32!(buf, fmt_paramsfromtransaction::DATASIZE)
        );

        //deserialize variable sized data
        if buf.len() < fmt_paramsfromtransaction::BASESIZE + data_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut data : Vec<u8> = vec![0u8; data_size as usize];
        data.copy_from_slice(&buf[fmt_paramsfromtransaction::BASESIZE..]);
        
        let ret = ParamsFromTransaction {
            from_address,
            to_address,
            value,
            data,
            transaction_hash,
        };
        Ok(ret)
    }
}

impl Serializable for ParamsFromBlockchain {
    fn serialize(msg: &ParamsFromBlockchain) -> Vec<u8> {
        // declare space to contains the bytes
        let mut ret :Vec<u8> = vec![0u8; fmt_paramsfromblockchain::BASESIZE];
        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_paramsfromblockchain::BLOCKNUM      | msg.this_block_number.to_le_bytes().as_slice(),
            fmt_paramsfromblockchain::PREVHASH      | msg.prev_block_hash.as_slice(),
            fmt_paramsfromblockchain::TIMESTAMP     | msg.timestamp.to_le_bytes().as_slice(),
            fmt_paramsfromblockchain::RANDOMBYTES   | msg.random_bytes.as_slice()
            => ret
        );
        ret
    }
}

impl Deserializable<ParamsFromBlockchain> for ParamsFromBlockchain {
    fn deserialize(buf: &[u8]) -> Result<ParamsFromBlockchain, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_paramsfromblockchain::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            this_block_number,
            prev_block_hash,
            timestamp,
            random_bytes
        ) = (
            deserialize_u64!(buf, fmt_paramsfromblockchain::BLOCKNUM),
            deserialize_32bytes!(buf, fmt_paramsfromblockchain::PREVHASH),
            deserialize_u32!(buf, fmt_paramsfromblockchain::TIMESTAMP),
            deserialize_32bytes!(buf, fmt_paramsfromblockchain::RANDOMBYTES)
        );

        let ret = ParamsFromBlockchain {
            this_block_number,
            prev_block_hash,
            timestamp,
            random_bytes,
        };
        Ok(ret)
    }
}



impl Serializable for CallData {
    fn serialize(msg: &Self) -> Vec<u8> {
        let method_name_size = msg.method_name.len();
        let arguments_size = msg.arguments.len();
        let mut ret = vec![0u8; fmt_calldata::METHOD_SIZE.0 + fmt_calldata::ARGS_SIZE.0 + method_name_size + arguments_size];
        
        serialize!(
            fmt_calldata::METHOD_SIZE  | (method_name_size as u32).to_le_bytes().as_slice(),
            fmt_calldata::ARGS_SIZE    | (arguments_size as u32).to_le_bytes().as_slice()
            => ret
        );
        ret[fmt_calldata::BASESIZE..fmt_calldata::BASESIZE+method_name_size].copy_from_slice(msg.method_name.as_bytes());
        ret[fmt_calldata::BASESIZE+method_name_size..fmt_calldata::BASESIZE+method_name_size+arguments_size].copy_from_slice(msg.arguments.as_slice());

        ret
    }
}

impl Deserializable<CallData> for CallData {
    fn deserialize(buf: &[u8]) -> Result<CallData, Error> {
        if buf.len() < fmt_calldata::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            method_name_size,
            arguments_size
        ) = (
            deserialize_u32!(buf, fmt_calldata::METHOD_SIZE),
            deserialize_u32!(buf, fmt_calldata::ARGS_SIZE)
        );

        let buf = &buf[fmt_calldata::BASESIZE..];
        if buf.len() < method_name_size as usize + arguments_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }

        let (buf_method_name, buf_arguments) = buf.split_at(method_name_size as usize);

        let method_name = match String::from_utf8(buf_method_name.to_vec()){
            Ok(m) => { m},
            Err(_) => { return Err(Error::new(ErrorKind::StringParseError))}
        };
        let arguments = buf_arguments.to_vec();

        Ok(CallData{
            method_name,
            arguments
        })
    }
}