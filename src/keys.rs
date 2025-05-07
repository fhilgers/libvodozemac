// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::slices::CErrorStr;
use crate::{CResult, ZST, boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_ED25519_PUBLIC_KEY_FROM_BYTES_JNI.into(),
            VODOZEMAC_ED25519_PUBLIC_KEY_TO_BYTES_JNI.into(),
            VODOZEMAC_ED25519_PUBLIC_KEY_VERIFY_JNI.into(),
            VODOZEMAC_ED25519_PUBLIC_KEY_FREE_JNI.into(),
            VODOZEMAC_CURVE25519_PUBLIC_KEY_FROM_BYTES_JNI.into(),
            VODOZEMAC_CURVE25519_PUBLIC_KEY_TO_BYTES_JNI.into(),
            VODOZEMAC_CURVE25519_PUBLIC_KEY_FREE_JNI.into(),
            VODOZEMAC_ED25519_SIGNATURE_FROM_BYTES_JNI.into(),
            VODOZEMAC_ED25519_SIGNATURE_TO_BYTES_JNI.into(),
            VODOZEMAC_ED25519_SIGNATURE_FREE_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_ed25519_public_key_from_bytes(
    bytes: &[u8; 32],
) -> Option<NonNull<Ed25519PublicKey>> {
    Some(boxed(Ed25519PublicKey::from_slice(bytes).ok()?))
}

#[ffi]
pub fn vodozemac_ed25519_public_key_to_bytes(key: &Ed25519PublicKey, bytes: &mut [u8; 32]) {
    bytes.copy_from_slice(key.as_bytes())
}

#[ffi]
#[sret]
pub fn vodozemac_ed25519_public_key_verify(
    key: &Ed25519PublicKey,
    #[expand] message: &[u8],
    signature: &Ed25519Signature,
) -> CResult<ZST, CErrorStr> {
    key.verify(message, signature)
        .map(Into::into)
        .map_err(Into::into)
        .into()
}

#[ffi]
pub fn vodozemac_ed25519_public_key_free(key: NonNull<Ed25519PublicKey>) {
    free(key)
}

#[ffi]
pub fn vodozemac_curve25519_public_key_from_bytes(
    bytes: &[u8; 32],
) -> NonNull<Curve25519PublicKey> {
    boxed(Curve25519PublicKey::from_bytes(*bytes))
}

#[ffi]
pub fn vodozemac_curve25519_public_key_to_bytes(key: &Curve25519PublicKey, bytes: &mut [u8; 32]) {
    bytes.copy_from_slice(key.as_bytes())
}

#[ffi]
pub fn vodozemac_curve25519_public_key_free(key: NonNull<Curve25519PublicKey>) {
    free(key)
}

#[ffi]
pub fn vodozemac_ed25519_signature_from_bytes(bytes: &[u8; 64]) -> NonNull<Ed25519Signature> {
    // TODO: fix in vodozemac
    boxed(Ed25519Signature::from_slice(bytes).expect("input is 64 bytes"))
}

#[ffi]
pub fn vodozemac_ed25519_signature_to_bytes(signature: &Ed25519Signature, bytes: &mut [u8; 64]) {
    bytes.copy_from_slice(&signature.to_bytes())
}

#[ffi]
pub fn vodozemac_ed25519_signature_free(signature: NonNull<Ed25519Signature>) {
    free(signature)
}
