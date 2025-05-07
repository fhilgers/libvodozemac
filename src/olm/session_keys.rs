// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::slices::CSlice;
use crate::{boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::Curve25519PublicKey;
use vodozemac::olm::SessionKeys;

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_OLM_SESSION_KEYS_FREE_JNI.into(),
            VODOZEMAC_OLM_SESSION_KEYS_IDENTITY_KEY_JNI.into(),
            VODOZEMAC_OLM_SESSION_KEYS_BASE_KEY_JNI.into(),
            VODOZEMAC_OLM_SESSION_KEYS_ONE_TIME_KEY_JNI.into(),
            VODOZEMAC_OLM_SESSION_KEYS_SESSION_ID_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_olm_session_keys_free(session_keys: NonNull<SessionKeys>) {
    free(session_keys)
}

#[ffi]
pub fn vodozemac_olm_session_keys_identity_key(
    session_keys: &SessionKeys,
) -> NonNull<Curve25519PublicKey> {
    boxed(session_keys.identity_key)
}

#[ffi]
pub fn vodozemac_olm_session_keys_base_key(
    session_keys: &SessionKeys,
) -> NonNull<Curve25519PublicKey> {
    boxed(session_keys.base_key)
}

#[ffi]
pub fn vodozemac_olm_session_keys_one_time_key(
    session_keys: &SessionKeys,
) -> NonNull<Curve25519PublicKey> {
    boxed(session_keys.one_time_key)
}

#[ffi]
#[sret]
pub fn vodozemac_olm_session_keys_session_id(session_keys: &SessionKeys) -> CSlice<u8> {
    session_keys.session_id().into()
}
