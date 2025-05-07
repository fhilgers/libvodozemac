// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::sas::Sas;
use crate::{boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use parking_lot::RwLock;
use std::ptr::NonNull;
use vodozemac::sas::EstablishedSas;
use vodozemac::{Curve25519PublicKey, sas};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_SAS_SAS_NEW_JNI.into(),
            VODOZEMAC_SAS_SAS_PUBLIC_KEY_JNI.into(),
            VODOZEMAC_SAS_SAS_DIFFIE_HELLMAN_JNI.into(),
            VODOZEMAC_SAS_SAS_FREE_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_sas_sas_new() -> NonNull<Sas> {
    boxed(RwLock::new(Some(sas::Sas::new())))
}

#[ffi]
pub fn vodozemac_sas_sas_public_key(sas: &Sas) -> NonNull<Curve25519PublicKey> {
    boxed(sas.read().as_ref().expect("sas is not used").public_key())
}

#[ffi]
pub fn vodozemac_sas_sas_diffie_hellman(
    sas: &Sas,
    their_public_key: &Curve25519PublicKey,
) -> Option<NonNull<EstablishedSas>> {
    Some(boxed(
        sas.write()
            .take()
            .expect("not used")
            .diffie_hellman(*their_public_key)
            .ok()?,
    ))
}

#[ffi]
pub fn vodozemac_sas_sas_free(sas: NonNull<Sas>) {
    free(sas)
}
