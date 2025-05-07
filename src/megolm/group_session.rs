// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::megolm::GroupSession;
use crate::slices::{CErrorStr, CSlice};
use crate::{
    CResult::{self},
    boxed, free,
};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::str;
use vodozemac::megolm;
use vodozemac::megolm::{GroupSessionPickle, MegolmMessage, SessionConfig, SessionKey};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_MEGOLM_GROUP_SESSION_NEW_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_FREE_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_SESSION_ID_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_MESSAGE_INDEX_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_SESSION_CONFIG_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_ENCRYPT_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_SESSION_KEY_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_PICKLE_JNI.into(),
            VODOZEMAC_MEGOLM_GROUP_SESSION_FROM_PICKLE_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_megolm_group_session_new(config: &SessionConfig) -> NonNull<GroupSession> {
    boxed(RwLock::new(megolm::GroupSession::new(*config)))
}

#[ffi]
pub fn vodozemac_megolm_group_session_free(group_session: NonNull<GroupSession>) {
    free(group_session)
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_group_session_session_id(group_session: &GroupSession) -> CSlice<u8> {
    group_session.read().session_id().into()
}

#[ffi]
pub fn vodozemac_megolm_group_session_message_index(group_session: &GroupSession) -> u32 {
    group_session.read().message_index()
}

#[ffi]
pub fn vodozemac_megolm_group_session_session_config(
    group_session: &GroupSession,
) -> NonNull<SessionConfig> {
    boxed(group_session.read().session_config())
}

#[ffi]
pub fn vodozemac_megolm_group_session_encrypt(
    group_session: &GroupSession,
    #[expand] plaintext: &[u8],
) -> NonNull<MegolmMessage> {
    boxed(group_session.write().encrypt(plaintext))
}

#[ffi]
pub fn vodozemac_megolm_group_session_session_key(
    group_session: &GroupSession,
) -> NonNull<SessionKey> {
    boxed(group_session.read().session_key())
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_group_session_pickle(
    group_session: &GroupSession,
    pickle_key: &[u8; 32],
) -> CSlice<u8> {
    group_session.read().pickle().encrypt(pickle_key).into()
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_group_session_from_pickle(
    #[expand] ciphertext: &[u8],
    pickle_key: &[u8; 32],
) -> CResult<NonNull<GroupSession>, CErrorStr> {
    let ciphertext = str::from_utf8(ciphertext).expect("valid utf8");

    GroupSessionPickle::from_encrypted(ciphertext, pickle_key)
        .map(megolm::GroupSession::from_pickle)
        .map(RwLock::new)
        .map(boxed)
        .map_err(Into::into)
        .into()
}
