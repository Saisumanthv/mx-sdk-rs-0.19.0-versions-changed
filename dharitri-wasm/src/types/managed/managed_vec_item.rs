use dharitri_codec::{NestedDecode, NestedEncode};

use crate::{
    abi::TypeAbi,
    api::{Handle, ManagedTypeApi},
};

use super::{BigInt, BigUint, ManagedAddress, ManagedBuffer, ManagedType};

/// Types that implement this trait can be items inside a `ManagedVec`.
/// All these types need a payload, i.e a representation that gets stored
/// in the underlying managed buffer.
/// Not all data needs to be stored as payload, for instance for most managed types
/// the payload is just the handle, whereas the mai ndata is kept by the VM.
pub trait ManagedVecItem<M: ManagedTypeApi>: NestedDecode + NestedEncode + TypeAbi {
    /// Size of the data stored in the underlying `ManagedBuffer`.
    const PAYLOAD_SIZE: usize;

    /// If false, then the encoding of the item is identical to the payload,
    /// and no further conversion is necessary
    /// (the underlying buffer can be used as-is during serialization).
    /// True for all managed types, but false for basic types (like `u32`).
    const NEEDS_RESERIALIZATION: bool;

    fn from_byte_reader<Reader: FnMut(&mut [u8])>(api: M, reader: Reader) -> Self;

    fn to_byte_writer<R, Writer: FnMut(&[u8]) -> R>(&self, writer: Writer) -> R;
}

macro_rules! impl_int {
    ($ty:ident, $payload_size:expr) => {
        impl<M: ManagedTypeApi> ManagedVecItem<M> for $ty {
            const PAYLOAD_SIZE: usize = $payload_size;
            const NEEDS_RESERIALIZATION: bool = false;

            fn from_byte_reader<Reader: FnMut(&mut [u8])>(_api: M, mut reader: Reader) -> Self {
                let mut arr: [u8; $payload_size] = [0u8; $payload_size];
                reader(&mut arr[..]);
                $ty::from_be_bytes(arr)
            }

            fn to_byte_writer<R, Writer: FnMut(&[u8]) -> R>(&self, mut writer: Writer) -> R {
                let bytes = self.to_be_bytes();
                writer(&bytes)
            }
        }
    };
}

impl_int! {u32, 4}
impl_int! {i32, 4}

macro_rules! impl_managed_type {
    ($ty:ident) => {
        impl<M: ManagedTypeApi> ManagedVecItem<M> for $ty<M> {
            const PAYLOAD_SIZE: usize = 4;
            const NEEDS_RESERIALIZATION: bool = true;

            fn from_byte_reader<Reader: FnMut(&mut [u8])>(api: M, reader: Reader) -> Self {
                let handle = Handle::from_byte_reader(api.clone(), reader);
                $ty::from_raw_handle(api, handle)
            }

            fn to_byte_writer<R, Writer: FnMut(&[u8]) -> R>(&self, writer: Writer) -> R {
                <Handle as ManagedVecItem<M>>::to_byte_writer(&self.get_raw_handle(), writer)
            }
        }
    };
}

impl_managed_type! {ManagedBuffer}
impl_managed_type! {BigUint}
impl_managed_type! {BigInt}
impl_managed_type! {ManagedAddress}
