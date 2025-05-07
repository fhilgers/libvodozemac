// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::olm::session::OlmMessage;
use crate::slices::{CErrorStr, CSlice};
use crate::{CResult, boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::olm::{Message, SessionKeys};
use vodozemac::{Curve25519PublicKey, olm};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_OLM_MESSAGE_FREE_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_RATCHET_KEY_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_CHAIN_INDEX_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_CIPHERTEXT_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_VERSION_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_MAC_TRUNCATED_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_TO_BYTES_JNI.into(),
            VODOZEMAC_OLM_MESSAGE_FROM_BYTES_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_olm_message_free(message: NonNull<Message>) {
    free(message)
}

#[ffi]
pub fn vodozemac_olm_message_ratchet_key(message: &Message) -> NonNull<Curve25519PublicKey> {
    boxed(message.ratchet_key())
}

#[ffi]
pub fn vodozemac_olm_message_chain_index(message: &Message) -> u64 {
    message.chain_index()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_message_ciphertext(message: &Message) -> CSlice<u8> {
    message.ciphertext().to_vec().into()
}

#[ffi]
pub fn vodozemac_olm_message_version(message: &Message) -> u32 {
    message.version() as u32
}

#[ffi]
pub fn vodozemac_olm_message_mac_truncated(message: &Message) -> u32 {
    message.mac_truncated().into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_message_to_bytes(
    message: &Message,
    session_keys: Option<&SessionKeys>,
) -> CSlice<u8> {
    olm::OlmMessage::from(OlmMessage::new(message, session_keys))
        .to_parts()
        .1
        .into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_message_from_bytes(
    message_type: u32,
    #[expand] bytes: &[u8],
) -> CResult<OlmMessage, CErrorStr> {
    olm::OlmMessage::from_parts(message_type as usize, bytes)
        .map(Into::into)
        .map_err(Into::into)
        .into()
}
