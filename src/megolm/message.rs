// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::slices::{CErrorStr, CSlice};
use crate::{CResult, boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::Ed25519Signature;
use vodozemac::megolm::MegolmMessage;

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_MEGOLM_MESSAGE_FREE_JNI.into(),
            VODOZEMAC_MEGOLM_MESSAGE_CIPHERTEXT_JNI.into(),
            VODOZEMAC_MEGOLM_MESSAGE_INDEX_JNI.into(),
            VODOZEMAC_MEGOLM_MESSAGE_MAC_JNI.into(),
            VODOZEMAC_MEGOLM_MESSAGE_SIGNATURE_JNI.into(),
            VODOZEMAC_MEGOLM_MESSAGE_TO_BYTES_JNI.into(),
            VODOZEMAC_MEGOLM_MESSAGE_FROM_BYTES_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_megolm_message_free(message: NonNull<MegolmMessage>) {
    free(message)
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_message_ciphertext(message: &MegolmMessage) -> CSlice<u8> {
    message.ciphertext().to_vec().into()
}

#[ffi]
pub fn vodozemac_megolm_message_index(message: &MegolmMessage) -> u32 {
    message.message_index()
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_message_mac(message: &MegolmMessage) -> CSlice<u8> {
    message.mac().to_vec().into()
}

#[ffi]
pub fn vodozemac_megolm_message_signature(message: &MegolmMessage) -> NonNull<Ed25519Signature> {
    boxed(*message.signature())
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_message_to_bytes(message: &MegolmMessage) -> CSlice<u8> {
    message.to_bytes().into()
}

#[ffi]
#[sret]
pub fn vodozemac_megolm_message_from_bytes(
    #[expand] bytes: &[u8],
) -> CResult<NonNull<MegolmMessage>, CErrorStr> {
    MegolmMessage::from_bytes(bytes)
        .map(boxed)
        .map_err(Into::into)
        .into()
}
