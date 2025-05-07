// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::sas::Mac;

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_SAS_MAC_FREE_JNI.into(),
            VODOZEMAC_SAS_MAC_AS_BYTES_JNI.into(),
            VODOZEMAC_SAS_MAC_FROM_SLICE_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_sas_mac_free(mac: NonNull<Mac>) {
    free(mac)
}

#[ffi]
pub fn vodozemac_sas_mac_as_bytes(mac: &Mac, bytes_out: &mut [u8; 32]) {
    bytes_out.copy_from_slice(mac.as_bytes())
}

#[ffi]
pub fn vodozemac_sas_mac_from_slice(bytes: &[u8; 32]) -> NonNull<Mac> {
    boxed(Mac::from_slice(&bytes[..]))
}
