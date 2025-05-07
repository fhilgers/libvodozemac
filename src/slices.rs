// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::free;
use macros::ffi;
use std::error::Error;
use std::ffi::c_void;
use std::ptr::{NonNull, slice_from_raw_parts_mut};

pub type Slice<T> = Box<[T]>;
pub type ByteSlice = Slice<u8>;
pub type PtrSlice<T> = Slice<NonNull<T>>;
pub type OpaquePtrSlice = PtrSlice<c_void>;

#[ffi]
pub fn vodozemac_ptr_slice_len(slice: &OpaquePtrSlice) -> u32 {
    slice.len() as u32
}

#[ffi]
pub fn vodozemac_ptr_slice_free(slice: NonNull<OpaquePtrSlice>) {
    free(slice);
}

#[ffi]
pub fn vodozemac_ptr_slice_copy_into(
    slice: &OpaquePtrSlice,
    offset: u32,
    slice_out: Option<NonNull<NonNull<c_void>>>,
    length: u32,
) {
    if length == 0 {
        return;
    }
    let Some(slice_out) = slice_out else { return };
    let slice_out = unsafe { &mut *slice_from_raw_parts_mut(slice_out.as_ptr(), length as usize) };

    slice_out.copy_from_slice(&slice[offset as usize..]);
}

#[ffi]
pub fn vodozemac_byte_slice_len(slice: &ByteSlice) -> u32 {
    slice.len() as u32
}

#[ffi]
pub fn vodozemac_byte_slice_free(slice: NonNull<ByteSlice>) {
    free(slice);
}

#[ffi]
pub fn vodozemac_byte_slice_copy_into(
    slice: &ByteSlice,
    offset: u32,
    slice_out: Option<NonNull<u8>>,
    length: u32,
) {
    if length == 0 {
        return;
    }
    let Some(slice_out) = slice_out else { return };

    let slice_out = unsafe { &mut *slice_from_raw_parts_mut(slice_out.as_ptr(), length as usize) };
    slice_out.copy_from_slice(&slice[offset as usize..]);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CSlice<T> {
    pub ptr: NonNull<T>,
    pub len: usize,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct CErrorStr(pub CSlice<u8>);

impl<T> From<Vec<T>> for CSlice<T> {
    fn from(value: Vec<T>) -> Self {
        let raw = Box::into_raw(value.into_boxed_slice());
        CSlice {
            ptr: unsafe { NonNull::new_unchecked(raw as _) },
            len: raw.len(),
        }
    }
}

impl From<String> for CSlice<u8> {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl<T: Error> From<T> for CErrorStr {
    fn from(value: T) -> Self {
        CErrorStr(value.to_string().into())
    }
}
