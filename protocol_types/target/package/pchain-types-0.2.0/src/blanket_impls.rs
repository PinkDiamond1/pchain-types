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
use crate::{Serializable, Deserializable, Error, ErrorKind};

impl Serializable for u32 {
    fn serialize(msg: &Self) -> Vec<u8> {
        let mut buf = [0u8; mem::size_of::<u32>()];
        buf.copy_from_slice(msg.to_le_bytes().as_slice());
        buf.to_vec()
    }
}

impl Deserializable<u32> for u32 {
    fn deserialize(buf: &[u8]) -> Result<u32, Error> {
        if buf.len() != mem::size_of::<u32>() {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut bs = [0u8; mem::size_of::<u32>()];
        bs.copy_from_slice(buf);
        Ok(u32::from_le_bytes(bs))
    }
}

impl Serializable for u64 {
    fn serialize(msg: &Self) -> Vec<u8> {
        let mut buf = [0u8; mem::size_of::<u64>()];
        buf.copy_from_slice(msg.to_le_bytes().as_slice());
        buf.to_vec()
    }
}

impl Deserializable<u64> for u64 {
    fn deserialize(buf: &[u8]) -> Result<u64, Error> {
        if buf.len() != mem::size_of::<u64>() {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut bs = [0u8; mem::size_of::<u64>()];
        bs.copy_from_slice(buf);
        Ok(u64::from_le_bytes(bs))
    }
}

impl Serializable for Vec<u8> {
    fn serialize(msg: &Self) -> Vec<u8> {
        msg.clone()
    }
}

impl Deserializable<Vec<u8>> for Vec<u8> {
    fn deserialize(buf: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(buf.to_vec())
    }
}

impl<T> Serializable for Option<T> where T: Serializable{
    fn serialize(msg: &Self) -> Vec<u8> {
        // 1 byte for true/false
        if msg.is_none() {
            [0u8].to_vec()
        } else {
            let serialized_bs = T::serialize(msg.as_ref().unwrap());
            [[1u8].to_vec(), serialized_bs].concat()
        }
    }
}

impl<T> Deserializable<Option<T>> for Option<T> where T: Deserializable<T> {
    fn deserialize(buf: &[u8]) -> Result<Option<T>, Error> {
        if buf.len() == 0 {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        Ok(
            if buf[0] == 0 {
                None
            } else {
                Some(T::deserialize(&buf[1..])?)
            }
        )
    }
}

impl<T1,T2, T3> Serializable for (T1,T2, T3) where T1: Serializable, T2: Serializable, T3: Serializable {
    fn serialize(msg: &Self) -> Vec<u8> {
        let serialzied_1 = T1::serialize(&msg.0);
        let serialzied_2 = T2::serialize(&msg.1);
        let serialzied_3 = T3::serialize(&msg.2);
        [
            (serialzied_1.len() as u32).to_le_bytes().to_vec(),
            (serialzied_2.len() as u32).to_le_bytes().to_vec(),
            (serialzied_3.len() as u32).to_le_bytes().to_vec(),
            serialzied_1,
            serialzied_2,
            serialzied_3
        ].concat()
    }
}
impl<T1,T2,T3> Deserializable<(T1,T2,T3)> for (T1,T2,T3) where T1: Deserializable<T1>, T2: Deserializable<T2>,T3: Deserializable<T3> {
    fn deserialize(buf: &[u8]) -> Result<(T1, T2, T3), Error> {
        if buf.len() < 3*mem::size_of::<u32>() {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let size_1 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];
        let size_2 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];
        let size_3 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];
        if buf.len() != size_1 + size_2 + size_3{
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let (serialized_1, serialized_2) = buf.split_at(size_1);
        let (serialized_2, serialized_3) = serialized_2.split_at(size_2);
        Ok((
            T1::deserialize(serialized_1)?,
            T2::deserialize(serialized_2)?,
            T3::deserialize(serialized_3)?
        ))
    }
}

impl<T1,T2> Serializable for (T1,T2) where T1: Serializable, T2: Serializable {
    fn serialize(msg: &Self) -> Vec<u8> {
        let serialzied_1 = T1::serialize(&msg.0);
        let serialzied_2 = T2::serialize(&msg.1);
        [
            (serialzied_1.len() as u32).to_le_bytes().to_vec(),
            (serialzied_2.len() as u32).to_le_bytes().to_vec(),
            serialzied_1,
            serialzied_2
        ].concat()
    }
}

impl<T1,T2> Deserializable<(T1,T2)> for (T1,T2) where T1: Deserializable<T1>, T2: Deserializable<T2> {
    fn deserialize(buf: &[u8]) -> Result<(T1, T2), Error> {
        if buf.len() < 2*mem::size_of::<u32>() {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        
        let size_1 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];

        let size_2 = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;
        let buf = &buf[mem::size_of::<u32>()..];

        if buf.len() != size_1 + size_2 {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }

        let (serialized_1, serialized_2) = buf.split_at(size_1);
        
        Ok((
            T1::deserialize(serialized_1)?,
            T2::deserialize(serialized_2)?
        ))
    }
}

/// Implementation of generic type in Vec. The serialization scheme follows Length-Value pattern.
impl<T> Serializable for Vec<T> where T: Serializable{
    fn serialize(msgs: &Self) -> Vec<u8> {
        let num_of_msg_bytes = (msgs.len() as u32).to_le_bytes().to_vec();
        let mut msg_size_bytes = vec![];
        let mut msg_data_bytes = vec![];
        msgs.iter().for_each(|msg|{
            let serialized = T::serialize(&msg);
            let serialized_size = serialized.len() as u32;
            msg_size_bytes.push(serialized_size.to_le_bytes().to_vec());
            msg_data_bytes.push(serialized);
        });
        [num_of_msg_bytes, msg_size_bytes.concat(), msg_data_bytes.concat()].concat()
    }
}

/// Implementation of generic type in Vec. The serialization scheme follows Length-Value pattern.
impl<T> Deserializable<Vec<T>> for Vec<T> where T: Deserializable<T> {
    fn deserialize(buf: &[u8]) -> Result<Vec<T>, Error> {
        // get the number of message
        if buf.len() < mem::size_of::<u32>() {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }

        let num_of_msg = deserialize_u32!(buf, (mem::size_of::<u32>(), 0)) as usize;

        // get sizes of msg
        let buf = &buf[(mem::size_of::<u32>() as usize) ..];
        let sizes_len = num_of_msg * mem::size_of::<u32>();
        if buf.len() < sizes_len {
            return Err(Error::new(ErrorKind::IncorrectLength));
        }
        let mut pos_size = 0usize;

        // get msgs
        let mut all_deserialized :Vec<T> = vec![];
        let mut pos_data = 0usize;
        let buf_data = &buf[sizes_len ..];
        
        while pos_size < sizes_len {
            let deserialized_size ={
                let mut bs = [0u8; mem::size_of::<u32>()];
                bs.copy_from_slice(& buf[pos_size..pos_size+mem::size_of::<u32>()]);
                u32::from_le_bytes(bs)
            } as usize;
            if buf_data.len() < pos_data + deserialized_size {
                return Err(Error::new(ErrorKind::IncorrectLength));
            }
            let deserialized = T::deserialize(&buf_data[pos_data..pos_data+deserialized_size])?;
            all_deserialized.push(deserialized);

            pos_size += mem::size_of::<u32>();
            pos_data += deserialized_size;
        }
        Ok(all_deserialized)
    }
}
