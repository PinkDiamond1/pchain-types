use std::convert::TryFrom;
use std::mem;
use crate::encodings::{fmt_transaction, fmt_event, fmt_receipt, serialize, deserialize_u32, deserialize_u64, fmt_transactiondata_contractdeployment, deserialize_32bytes, deserialize_64bytes};
use crate::{crypto, Serializable, Deserializable, Error, ErrorKind, receipt_status_codes, ReceiptStatusCode};

/// Transactions are authenticated, non-repudiable messages produced by external accounts 
/// to authorize blockchain state transitions, either through token transfer or smart contract
/// execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub from_address : crypto::PublicAddress,
    pub to_address : crypto::PublicAddress,
    pub value : u64,
    pub tip : u64,
    pub gas_limit : u64,
    pub gas_price : u64,
    pub data : Vec<u8>,
    pub n_txs_on_chain_from_address : u64,
    pub hash : crypto::Sha256Hash,
    pub signature : crypto::Signature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[deprecated="No necessary use cases. Please consider using Vec<Vec<u8> or Vec<Transaction> directly."]
pub struct Transactions {
    pub transactions : Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionDataContractDeployment {
    pub contract_code : Vec<u8>,
    pub contract_init_arguments : Vec<u8>
}

/// Events are messages produced by smart contract executions that are persisted on the blockchain
/// in a cryptographically-provable way. Events produced by transactions that call smart contracts
/// are stored in the `events` field of a Block in the order in which they are emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub topic : Vec<u8>,
    pub value : Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub status_code: receipt_status_codes::ReceiptStatusCode,
    pub gas_consumed: u64,
    pub return_value: Vec<u8>,
    pub events: Vec<Event>,
}

impl Transaction {
    #[inline]
    pub fn size_of(msg :&Transaction) -> usize {
        msg.data.len() + fmt_transaction::BASESIZE
    }

    #[inline]
    pub(crate) fn size_from_slice(buf:&[u8]) -> Result<usize, Error> {
        if buf.len() < fmt_transaction::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let data_size = deserialize_u32!(buf, fmt_transaction::DATASIZE);
        Ok(fmt_transaction::BASESIZE + data_size as usize)
    }
    
    pub(crate) fn serialize_to_slice(msg: &Transaction, ret :&mut[u8]) -> usize {
        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_transaction::FROMADDRESS | msg.from_address.as_slice(),
            fmt_transaction::TOADDRESS   | msg.to_address.as_slice(),
            fmt_transaction::VALUE       | msg.value.to_le_bytes().as_slice(),
            fmt_transaction::TIP         | msg.tip.to_le_bytes().as_slice(),
            fmt_transaction::GASLIMIT    | msg.gas_limit.to_le_bytes().as_slice(),
            fmt_transaction::GASPRICE    | msg.gas_price.to_le_bytes().as_slice(),
            fmt_transaction::NUMTX       | msg.n_txs_on_chain_from_address.to_le_bytes().as_slice(),
            fmt_transaction::HASH        | msg.hash.as_slice(),
            fmt_transaction::SIGNATURE   | msg.signature.as_slice(),
            fmt_transaction::DATASIZE    | (msg.data.len() as u32).to_le_bytes().as_slice()
            => ret
        );

        // serialize the variable sized data
        ret[fmt_transaction::BASESIZE..fmt_transaction::BASESIZE+msg.data.len()].copy_from_slice(msg.data.as_slice());

        Self::size_of(msg)
    }

}

impl Serializable for Transaction {
    fn serialize(msg: &Transaction) -> Vec<u8> {
        let mut ret :Vec<u8> = vec![0u8; Self::size_of(msg)];
        Self::serialize_to_slice(msg, &mut ret);
        ret
    }
}


impl Deserializable<Transaction> for Transaction {
    fn deserialize(buf: &[u8]) -> Result<Transaction, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_transaction::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            from_address,
            to_address,
            value,
            tip,
            gas_limit,
            gas_price,
            n_txs_on_chain_from_address,
            hash,
            signature,
            data_size
        ) = (
            deserialize_32bytes!(buf, fmt_transaction::FROMADDRESS),
            deserialize_32bytes!(buf, fmt_transaction::TOADDRESS),
            deserialize_u64!(buf, fmt_transaction::VALUE),
            deserialize_u64!(buf, fmt_transaction::TIP),
            deserialize_u64!(buf, fmt_transaction::GASLIMIT),
            deserialize_u64!(buf, fmt_transaction::GASPRICE),
            deserialize_u64!(buf, fmt_transaction::NUMTX),
            deserialize_32bytes!(buf, fmt_transaction::HASH),
            deserialize_64bytes!(buf, fmt_transaction::SIGNATURE),
            deserialize_u32!(buf, fmt_transaction::DATASIZE)
        );

        //deserialize variable sized data
        if buf.len() < fmt_transaction::BASESIZE + data_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut data : Vec<u8> = vec![0u8; data_size as usize];
        data.copy_from_slice(&buf[fmt_transaction::BASESIZE..]);
        
        Ok(Transaction {
            from_address,
            to_address,
            value,
            tip,
            gas_limit,
            gas_price,
            data,
            n_txs_on_chain_from_address,
            hash,
            signature
        })
    }
}

#[allow(deprecated)]
impl Serializable for Transactions {
    fn serialize(msg: & Transactions) -> Vec<u8> {
        let transactions :Vec<u8> = msg.transactions.iter().flat_map(|tx|{
            let serialized_tx = Transaction::serialize(&tx);
            let tx_size = serialized_tx.len();
            let mut entry_bytes = vec![0u8; mem::size_of::<u32>() + tx_size];
            entry_bytes[0..mem::size_of::<u32>()].copy_from_slice((tx_size as u32).to_le_bytes().as_slice());
            entry_bytes[mem::size_of::<u32>()..].copy_from_slice(serialized_tx.as_slice());
            entry_bytes
        }).collect();
        transactions
    }
}

#[allow(deprecated)]
impl Deserializable<Transactions> for Transactions {
    fn deserialize(buf: &[u8]) -> Result<Transactions, Error> {
        let mut transactions :Vec<Transaction> = vec![];
        let mut pos = 0usize;
        while pos < buf.len() {
            // get the size of transaction
            if buf.len() < pos + mem::size_of::<u32>() {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            let tx_size ={
                let mut bs = [0u8; mem::size_of::<u32>()];
                bs.copy_from_slice(& buf[pos..pos+mem::size_of::<u32>()]);
                u32::from_le_bytes(bs)
            };

            // get the transaction
            if buf.len() < pos + mem::size_of::<u32>() + tx_size as usize {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            let tx = Transaction::deserialize(&buf[pos+mem::size_of::<u32>()..pos+mem::size_of::<u32>()+tx_size as usize])?;
            transactions.push(tx);
            pos += mem::size_of::<u32>() + tx_size as usize;
        }

        Ok(Transactions {
            transactions
        })
    }
}


impl Serializable for TransactionDataContractDeployment {
    fn serialize(msg: &TransactionDataContractDeployment) -> Vec<u8> {
        let contract_code_size = msg.contract_code.len();
        let init_args_size = msg.contract_init_arguments.len();
        let mut ret :Vec<u8> = vec![0u8; fmt_transactiondata_contractdeployment::BASESIZE + contract_code_size + init_args_size];
        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_transactiondata_contractdeployment::CODESIZE  | (contract_code_size as u32).to_le_bytes().as_slice(),
            fmt_transactiondata_contractdeployment::ARGSSIZE  | (init_args_size as u32).to_le_bytes().as_slice()
            => ret
        );

        // serialize the variable sized data
        let mut pos = fmt_transactiondata_contractdeployment::BASESIZE;
        ret[pos..pos+contract_code_size].copy_from_slice(msg.contract_code.as_slice());
        pos += contract_code_size;
        ret[pos..pos+init_args_size].copy_from_slice(msg.contract_init_arguments.as_slice());

        ret
    }
}


impl Deserializable<TransactionDataContractDeployment> for TransactionDataContractDeployment {

    fn deserialize(buf: &[u8]) -> Result<TransactionDataContractDeployment, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_transactiondata_contractdeployment::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            code_size,
            args_size
         ) = (
            deserialize_u32!(buf, fmt_transactiondata_contractdeployment::CODESIZE),
            deserialize_u32!(buf, fmt_transactiondata_contractdeployment::ARGSSIZE)
        );

        // deserialize variable sized data
        if buf.len() < fmt_transactiondata_contractdeployment::BASESIZE + code_size as usize + args_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut pos = fmt_event::BASESIZE;
        let mut contract_code : Vec<u8> = vec![0u8; code_size as usize];
        contract_code.copy_from_slice(&buf[pos..pos+code_size as usize]);
        pos += code_size as usize;

        let mut contract_init_arguments : Vec<u8> = vec![0u8; args_size as usize];
        contract_init_arguments.copy_from_slice(&buf[pos..pos+args_size as usize]);

        Ok(TransactionDataContractDeployment {
            contract_code,
            contract_init_arguments
        })

    }
}

impl Event {
    #[inline]
    pub fn size_of(msg :&Event) -> usize {
        msg.topic.len() + msg.value.len() + fmt_event::BASESIZE
    }

    #[inline]
    pub(crate) fn size_from_slice(buf:&[u8]) -> Result<usize, Error> {
        if buf.len() < fmt_event::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let topic_size = deserialize_u32!(buf, fmt_event::TOPICSIZE);
        let value_size = deserialize_u32!(buf, fmt_event::VALUESIZE);
        Ok(fmt_event::BASESIZE + topic_size as usize + value_size as usize)
    }
    
    pub(crate) fn serialize_to_slice(msg: &Event, ret :&mut[u8]) -> usize {
        let msg_topic_size = msg.topic.len();
        let msg_value_size = msg.value.len();

        // serialize fixed size data, the fields in msg, one-by-one
        serialize!(
            fmt_event::TOPICSIZE  | (msg_topic_size as u32).to_le_bytes().as_slice(),
            fmt_event::VALUESIZE  | (msg_value_size as u32).to_le_bytes().as_slice()
            => ret
        );

        // serialize the variable sized data
        let mut pos = fmt_event::BASESIZE;
        ret[pos..pos+msg_topic_size].copy_from_slice(msg.topic.as_slice());
        pos += msg_topic_size;
        ret[pos..pos+msg_value_size].copy_from_slice(msg.value.as_slice());

        Self::size_of(msg)
    }
}

impl Serializable for Event {

    fn serialize(msg: &Event) -> Vec<u8> {
        let mut ret :Vec<u8> = vec![0u8; Self::size_of(msg)];
        Self::serialize_to_slice(msg, &mut ret);
        ret
    }
}
impl Deserializable<Event> for Event {

    fn deserialize(buf: &[u8]) -> Result<Event, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_event::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (
            topic_size,
            value_size
         ) = (
            deserialize_u32!(buf, fmt_event::TOPICSIZE),
            deserialize_u32!(buf, fmt_event::VALUESIZE)
        );

        // deserialize variable sized data
        if buf.len() < fmt_event::BASESIZE + topic_size as usize + value_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut pos = fmt_event::BASESIZE;
        let mut topic : Vec<u8> = vec![0u8; topic_size as usize];
        topic.copy_from_slice(&buf[pos..pos+topic_size as usize]);
        pos += topic_size as usize;

        let mut value : Vec<u8> = vec![0u8; value_size as usize];
        value.copy_from_slice(&buf[pos..pos+value_size as usize]);

        Ok(Event {
            topic,
            value
        })
    }
}

impl Receipt {
    #[inline]
    pub(crate) fn size_of(msg :&Receipt) -> usize {
        let events_size :usize = msg.events.iter().map(|e|{
            Event::size_of(e)
        }).sum();
        msg.return_value.len() + events_size + fmt_receipt::BASESIZE
    }

    #[inline]
    pub(crate) fn size_from_slice(buf:&[u8]) -> Result<usize, Error> {
        if buf.len() < fmt_receipt::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let returnval_size = deserialize_u32!(buf, fmt_receipt::RETURNVALSIZE);
        let events_size = deserialize_u32!(buf, fmt_receipt::EVENTSSIZE);
        Ok(fmt_receipt::BASESIZE + returnval_size as usize + events_size as usize)
    }

    pub(crate) fn serialize_to_slice(msg: &Receipt, ret :&mut[u8]) -> usize {
        let msg_return_value_size = msg.return_value.len();
        let events_size :usize = msg.events.iter().map(|e|{
            Event::size_of(e)
        }).sum();

        // serialize fixed size data, the fields in msg, one-by-one
        ret[fmt_receipt::STATUSCODE.1] = {
            let status_code :u8 = msg.status_code.clone().into();
            status_code
        };
        serialize!(
            fmt_receipt::GASCONSUMED    | msg.gas_consumed.to_le_bytes().as_slice(),
            fmt_receipt::RETURNVALSIZE  | (msg_return_value_size as u32).to_le_bytes().as_slice(),
            fmt_receipt::EVENTSSIZE     | (events_size as u32).to_le_bytes().as_slice()
            => ret
        );

        // serialize the variable sized data
        let mut pos = fmt_receipt::BASESIZE;
        ret[pos..pos+msg_return_value_size].copy_from_slice(msg.return_value.as_slice());
        pos += msg_return_value_size;
        msg.events.iter().for_each(|e|{
            pos += Event::serialize_to_slice(e, &mut ret[pos..]);
        });

        Self::size_of(msg)
    }
}

impl Serializable for Receipt {
    
    fn serialize(msg: &Receipt) -> Vec<u8> {
        let mut ret :Vec<u8> = vec![0u8; Self::size_of(msg)];
        Self::serialize_to_slice(msg, &mut ret);
        ret
    }
}

impl Deserializable<Receipt> for Receipt {
    
    fn deserialize(buf: &[u8]) -> Result<Receipt, Error> {
        // deserialize fixed size data
        if buf.len() < fmt_receipt::BASESIZE {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let status_code = ReceiptStatusCode::try_from(buf[fmt_receipt::STATUSCODE.1] as u8)?;
        let (
            gas_consumed,
            msg_return_value_size,
            events_size,
        ) = (
            deserialize_u64!(buf, fmt_receipt::GASCONSUMED),
            deserialize_u32!(buf, fmt_receipt::RETURNVALSIZE),
            deserialize_u32!(buf, fmt_receipt::EVENTSSIZE),
        );

        // deserialize variable sized data
        if buf.len() < fmt_receipt::BASESIZE + msg_return_value_size as usize + events_size as usize {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut pos = fmt_receipt::BASESIZE;
        let mut return_value : Vec<u8> = vec![0u8; msg_return_value_size as usize];
        return_value.copy_from_slice(&buf[pos..pos+msg_return_value_size as usize]);
        pos += msg_return_value_size as usize;

        let mut events = vec![];
        while pos < buf.len() {
            let events_buf = &buf[pos..];
            let size_of_i_th_evt = Event::size_from_slice(&events_buf)?;
            events.push(
                Event::deserialize(&events_buf[..size_of_i_th_evt])?
            );
            pos += size_of_i_th_evt;
        }

        Ok(Receipt {
            gas_consumed,
            status_code,
            return_value,
            events
        })
    }
}
