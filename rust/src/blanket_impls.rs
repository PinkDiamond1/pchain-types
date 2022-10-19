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


use crate::{Serializable, Deserializable};


impl Serializable<u32> for u32 {}

impl Deserializable<u32> for u32 {}

impl Serializable<u64> for u64 {}

impl Deserializable<u64> for u64 {}

impl Serializable<Vec<u8>> for Vec<u8> {}
  
impl Deserializable<Vec<u8>> for Vec<u8> {}

impl<T: borsh::BorshSerialize> Serializable<Option<T>> for Option<T> where T: Serializable<T>{}

impl<T: borsh::BorshDeserialize> Deserializable<Option<T>> for Option<T> where T: Deserializable<T> {}

impl<T1 :borsh::BorshSerialize,T2: borsh::BorshSerialize, T3: borsh::BorshSerialize> Serializable<(T1,T2, T3)> for (T1,T2, T3) where T1: Serializable<T1>, T2: Serializable<T2>, T3: Serializable<T3> {}
impl<T1 :borsh::BorshDeserialize, T2: borsh::BorshDeserialize, T3: borsh::BorshDeserialize> Deserializable<(T1,T2,T3)> for (T1,T2,T3) where T1: Deserializable<T1>, T2: Deserializable<T2>,T3: Deserializable<T3> {   }

impl<T1 :borsh::BorshSerialize,T2: borsh::BorshSerialize> Serializable<(T1,T2)> for (T1,T2) where T1: Serializable<T1>, T2: Serializable<T2> {}
impl<T1 :borsh::BorshDeserialize,T2: borsh::BorshDeserialize> Deserializable<(T1,T2)> for (T1,T2) where T1: Deserializable<T1>, T2: Deserializable<T2> {}

/// Implementation of generic type in Vec. The serialization scheme follows Length-Value pattern.
impl<T :borsh::BorshSerialize> Serializable<Vec<T>> for Vec<T> where T: Serializable<T>{}

/// Implementation of generic type in Vec. The serialization scheme follows Length-Value pattern.
impl<T :borsh::BorshDeserialize> Deserializable<Vec<T>> for Vec<T> where T: Deserializable<T> {}