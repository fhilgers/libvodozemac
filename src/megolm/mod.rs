// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use jni::JNIEnv;
use jni::objects::JClass;
use parking_lot::RwLock;
use vodozemac::megolm;

mod exported_session_key;
mod group_session;
mod inbound_group_session;
mod message;
mod session_config;
mod session_key;

pub type GroupSession = RwLock<megolm::GroupSession>;
pub type InboundGroupSession = RwLock<megolm::InboundGroupSession>;

pub struct MegolmJniClasses<'local, 'a> {
    pub group_session: &'a JClass<'local>,
    pub inbound_group_session: &'a JClass<'local>,
    pub message: &'a JClass<'local>,
    pub session_config: &'a JClass<'local>,
    pub session_key: &'a JClass<'local>,
    pub exported_session_key: &'a JClass<'local>,
}

pub fn register_jni(
    env: &mut JNIEnv,
    MegolmJniClasses {
        group_session,
        inbound_group_session,
        message,
        session_config,
        session_key,
        exported_session_key,
    }: &MegolmJniClasses,
) -> jni::errors::Result<()> {
    group_session::register_jni(env, group_session)?;
    inbound_group_session::register_jni(env, inbound_group_session)?;
    message::register_jni(env, message)?;
    session_config::register_jni(env, session_config)?;
    session_key::register_jni(env, session_key)?;
    exported_session_key::register_jni(env, exported_session_key)?;

    Ok(())
}
