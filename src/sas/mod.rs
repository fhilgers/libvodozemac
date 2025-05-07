// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use jni::JNIEnv;
use jni::objects::JClass;
use parking_lot::RwLock;

pub mod established_sas;
pub mod mac;

#[allow(clippy::module_inception)]
pub mod sas;
pub mod sas_bytes;

pub type Sas = RwLock<Option<vodozemac::sas::Sas>>;

pub struct SasJniClasses<'local, 'a> {
    pub established_sas: &'a JClass<'local>,
    pub mac: &'a JClass<'local>,
    pub sas: &'a JClass<'local>,
    pub sas_bytes: &'a JClass<'local>,
}

pub fn register_jni(
    env: &mut JNIEnv,
    SasJniClasses {
        established_sas,
        mac,
        sas,
        sas_bytes,
    }: &SasJniClasses,
) -> jni::errors::Result<()> {
    established_sas::register_jni(env, established_sas)?;
    mac::register_jni(env, mac)?;
    sas::register_jni(env, sas)?;
    sas_bytes::register_jni(env, sas_bytes)?;

    Ok(())
}
