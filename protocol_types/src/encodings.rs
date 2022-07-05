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

macro_rules! define_format {
    ( $($fm:ident | $size:literal | $offset:literal),* => $out_size:ident ) =>{
        $(
            pub const $fm : (usize, usize)= ($size,$offset);
        )*

        pub const $out_size :usize = $($fm.0 +)* 0 ;

    };
}

/// Data format of Block Header
pub mod fmt_blockheader {
    // define format for BlockHeader
    define_format!(
        // field    | size  | offset
        ID          | 8     | 0,
        VERSION     | 8     | 8,
        TIMESTAMP   | 4     | 16,
        PREVHASH    | 32    | 20,
        BLOCKHASH   | 32    | 52,
        STATEHASH   | 32    | 84,
        TXSHASH     | 32    | 116,
        RECEIPTHASH | 32    | 148,
        PUBKEY      | 32    | 180,
        SIGNATURE   | 64    | 212
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of sizes on Block Data entries
pub mod fmt_sizes {
    define_format!(
        // field    | size  | offset
        TRANSACTION | 4     | 0, // unused
        RECEIPT     | 4     | 4
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of transactin entry
pub mod fmt_transaction {
    // define format for Transaction
    define_format!(
        // field    | size  | offset
        FROMADDRESS | 32    | 0, 
        TOADDRESS   | 32    | 32, 
        VALUE       | 8     | 64, 
        TIP         | 8     | 72, 
        GASLIMIT    | 8     | 80,
        GASPRICE    | 8     | 88,
        NUMTX       | 8     | 96,
        HASH        | 32    | 104,
        SIGNATURE   | 64    | 136,
        DATASIZE    | 4     | 200
        // DATA     | bytes with variable size
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of TransactionDataContractDeployment
pub mod fmt_transactiondata_contractdeployment {
    // define format for Transaction
    define_format!(
        // field    | size  | offset
        CODESIZE    | 4    | 0, 
        ARGSSIZE    | 4    | 4
        // CODE     | bytes with variable size
        // ARGS     | bytes with variable size
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of Event entry
pub mod fmt_event {
     // define format for Event
     define_format!(
        // field          | size  | offset
        TOPICSIZE         | 4     | 0,
        VALUESIZE         | 4     | 4
        // TOPIC          | bytes with variable size
        // VALUE          | bytes with variable size
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of Receipt entry
pub mod fmt_receipt {
    // define format for Receipt
    define_format!(
        // field        | size  | offset
        STATUSCODE      | 1     | 0,
        GASCONSUMED     | 8     | 1,
        RETURNVALSIZE   | 4     | 9,
        EVENTSSIZE      | 4     | 13
        // RETURNVAL    | bytes with variable size
        // EEVENTS      | bytes with variable size
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// Data format of Merkle Proof
pub mod fmt_merkleproof {
    // define format for MerkleProof
    define_format!(
        // field        | size  | offset
        ROOTHASH        | 32    | 0,
        TOTALLEAVES     | 4     | 32,
        LEAFINDSIZE     | 4     | 36,
        LEAFHASHSIZE    | 4     | 40,
        PROOFSIZE       | 4     | 44
        // LEAFIND      | bytes with variable size 
        // LEAFHASH     | bytes with variable size 
        // PROOF        | bytes with variable size 
        => BASESIZE // sum of size of fields with fixed size
    );
}

/// `serialize` is macro to copy slices of byte to destination buffer according to data size and offset
/// ### Example
/// ```no_run
/// serialize!(
///    fmt_data::THE_SLICE      | the_slice
///    => ret
/// );
/// ```
/// converts to 
/// ```no_run
/// ret[THE_SLICE.1..THE_SLICE.0].copy_from_slice(the_slice);
/// ```
macro_rules! serialize {
    ($( $t:ident::$fmt:ident | $val:expr),* => $ret:ident) => {
        $(
            $ret[$t::$fmt.1..$t::$fmt.1 + $t::$fmt.0].copy_from_slice($val);
        )*
    }
}
pub(crate) use serialize;

/// `deserialize_32bytes` is macro to copy from a slice of bytes to [u8; 32] according to data size and offset
macro_rules! deserialize_32bytes{
    ($buf:expr, $fmt:expr) => {
        {
            let mut bs = [0u8; 32];
            bs.copy_from_slice(& $buf[$fmt.1..$fmt.1+$fmt.0]);
            bs
        }
    }
}
pub(crate) use deserialize_32bytes;

/// `deserialize_64bytes` is macro to copy from a slice of bytes to [u8; 64] according to data size and offset
macro_rules! deserialize_64bytes{
    ($buf:expr, $fmt:expr) => {
        {
            let mut bs = [0u8; 64];
            bs.copy_from_slice(& $buf[$fmt.1..$fmt.1+$fmt.0]);
            bs
        }
    }
}
pub(crate) use deserialize_64bytes;

/// `deserialize_u64` is macro to copy vector of bytes to u64 according to data size and offset
/// ### Example
/// ```no_run
/// deserialize_u64!(buf , fmt_data::THE_U64);
/// ```
/// converts to 
/// ```no_run
/// {
///     let mut bs = [0u8; mem::size_of::<u64>()];
///     bs.copy_from_slice(&buf[fmt_data::THE_U64.1..fmt_data::THE_U64.1+fmt_data::THE_U64.0]);
///     u64::from_le_bytes(bs)
/// }
/// ```
macro_rules! deserialize_u64{
    ($buf:expr, $fmt:expr) => {
        {
            let mut bs = [0u8; mem::size_of::<u64>()];
            bs.copy_from_slice(& $buf[$fmt.1..$fmt.1+$fmt.0]);
            u64::from_le_bytes(bs)
        }
    };
}
pub(crate) use deserialize_u64;

/// `deserialize_u32` is macro to copy vector of bytes to u32 according to data size and offset
/// ### Example
/// ```no_run
/// deserialize_u32!(buf , fmt_data::THE_U32);
/// ```
/// converts to 
/// ```no_run
/// {
///     let mut bs = [0u8; mem::size_of::<u32>()];
///     bs.copy_from_slice(&buf[fmt_data::THE_U32.1..fmt_data::THE_U32.1+fmt_data::THE_U32.0]);
///     u32::from_le_bytes(bs)
/// }
/// ```
macro_rules! deserialize_u32{
    ($buf:expr, $fmt:expr) => {
        {
            let mut bs = [0u8; mem::size_of::<u32>()];
            bs.copy_from_slice(& $buf[$fmt.1..$fmt.1+$fmt.0]);
            u32::from_le_bytes(bs)
        }
    };
}
pub(crate) use deserialize_u32;