// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::slices::{CErrorStr, CSlice};
use crate::{CResult, boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::megolm::ExportedSessionKey;

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_MEGOLM_EXPORTED_SESSION_KEY_FREE_JNI.into(),
            VODOZEMAC_MEGOLM_EXPORTED_SESSION_KEY_TO_BYTES_JNI.into(),
            VODOZEMAC_MEGOLM_EXPORTED_SESSION_KEY_FROM_BYTES_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_megolm_exported_session_key_free(session_key: NonNull<ExportedSessionKey>) {
    free(session_key)
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_exported_session_key_to_bytes(
    session_key: &ExportedSessionKey,
) -> CSlice<u8> {
    session_key.to_bytes().into()
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_exported_session_key_from_bytes(
    #[expand] bytes: &[u8],
) -> CResult<NonNull<ExportedSessionKey>, CErrorStr> {
    ExportedSessionKey::from_bytes(bytes)
        .map(boxed)
        .map_err(Into::into)
        .into()
}
