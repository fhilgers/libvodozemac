// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use std::str;
use vodozemac::Curve25519PublicKey;
use vodozemac::sas::{EstablishedSas, Mac, SasBytes};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_SAS_ESTABLISHED_SAS_FREE_JNI.into(),
            VODOZEMAC_SAS_ESTABLISHED_SAS_BYTES_JNI.into(),
            VODOZEMAC_SAS_ESTABLISHED_SAS_CALCULATE_MAC_JNI.into(),
            VODOZEMAC_SAS_ESTABLISHED_SAS_VERIFY_MAC_JNI.into(),
            VODOZEMAC_SAS_ESTABLISHED_SAS_OUR_PUBLIC_KEY_JNI.into(),
            VODOZEMAC_SAS_ESTABLISHED_SAS_THEIR_PUBLIC_KEY_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_sas_established_sas_free(sas: NonNull<EstablishedSas>) {
    free(sas)
}

#[ffi]
pub fn vodozemac_sas_established_sas_bytes(
    sas: &EstablishedSas,
    #[expand] info: &[u8],
) -> NonNull<SasBytes> {
    boxed(sas.bytes(str::from_utf8(info).expect("valid utf8")))
}

#[ffi]
pub fn vodozemac_sas_established_sas_calculate_mac(
    sas: &EstablishedSas,
    #[expand] input: &[u8],
    #[expand] info: &[u8],
) -> NonNull<Mac> {
    let input = str::from_utf8(input).expect("valid utf8");
    let info = str::from_utf8(info).expect("valid utf8");

    boxed(sas.calculate_mac(input, info))
}

#[ffi]
pub fn vodozemac_sas_established_sas_verify_mac(
    sas: &EstablishedSas,
    #[expand] input: &[u8],
    #[expand] info: &[u8],
    tag: &Mac,
) -> u32 {
    let input = str::from_utf8(input).expect("valid utf8");
    let info = str::from_utf8(info).expect("valid utf8");

    sas.verify_mac(input, info, tag).is_ok().into()
}

#[ffi]
pub fn vodozemac_sas_established_sas_our_public_key(
    sas: &EstablishedSas,
) -> NonNull<Curve25519PublicKey> {
    boxed(sas.our_public_key())
}

#[ffi]
pub fn vodozemac_sas_established_sas_their_public_key(
    sas: &EstablishedSas,
) -> NonNull<Curve25519PublicKey> {
    boxed(sas.their_public_key())
}
