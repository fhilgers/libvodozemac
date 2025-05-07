// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use jni::JNIEnv;
use jni::objects::JClass;
use parking_lot::RwLock;

pub mod account;
pub mod message;
pub mod session;
pub mod session_config;
pub mod session_keys;

pub type Account = RwLock<vodozemac::olm::Account>;
pub type Session = RwLock<vodozemac::olm::Session>;

pub struct OlmJniClasses<'local, 'a> {
    pub account: &'a JClass<'local>,
    pub message: &'a JClass<'local>,
    pub session: &'a JClass<'local>,
    pub session_config: &'a JClass<'local>,
    pub session_keys: &'a JClass<'local>,
}

pub fn register_jni(
    env: &mut JNIEnv,
    OlmJniClasses {
        account,
        message,
        session,
        session_config,
        session_keys,
    }: &OlmJniClasses,
) -> jni::errors::Result<()> {
    account::register_jni(env, account)?;
    message::register_jni(env, message)?;
    session::register_jni(env, session)?;
    session_config::register_jni(env, session_config)?;
    session_keys::register_jni(env, session_keys)?;

    Ok(())
}
