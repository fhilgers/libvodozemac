// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::free;
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::sas::SasBytes;

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_SAS_SAS_BYTES_FREE_JNI.into(),
            VODOZEMAC_SAS_SAS_BYTES_EMOJI_INDICES_JNI.into(),
            VODOZEMAC_SAS_SAS_BYTES_DECIMALS_JNI.into(),
            VODOZEMAC_SAS_SAS_BYTES_AS_BYTES_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_sas_sas_bytes_free(sas_bytes: NonNull<SasBytes>) {
    free(sas_bytes)
}

#[ffi]
pub fn vodozemac_sas_sas_bytes_emoji_indices(
    sas_bytes: &SasBytes,
    emoji_indices_out: &mut [u8; 7],
) {
    emoji_indices_out.copy_from_slice(&sas_bytes.emoji_indices())
}

#[ffi]
pub fn vodozemac_sas_sas_bytes_decimals(sas_bytes: &SasBytes, decimals_out: &mut [u16; 3]) {
    let decimals = sas_bytes.decimals();
    decimals_out[0] = decimals.0;
    decimals_out[1] = decimals.1;
    decimals_out[2] = decimals.2;
}

#[ffi]
pub fn vodozemac_sas_sas_bytes_as_bytes(sas_bytes: &SasBytes, bytes_out: &mut [u8; 6]) {
    bytes_out.copy_from_slice(sas_bytes.as_bytes())
}
