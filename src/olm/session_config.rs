// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use std::ptr::NonNull;
use vodozemac::olm::SessionConfig;

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_OLM_SESSION_CONFIG_VERSION_1_JNI.into(),
            VODOZEMAC_OLM_SESSION_CONFIG_VERSION_2_JNI.into(),
            VODOZEMAC_OLM_SESSION_CONFIG_VERSION_JNI.into(),
            VODOZEMAC_OLM_SESSION_CONFIG_FREE_JNI.into(),
        ],
    )
}

#[ffi]
pub fn vodozemac_olm_session_config_version_1() -> NonNull<SessionConfig> {
    boxed(SessionConfig::version_1())
}

#[ffi]
pub fn vodozemac_olm_session_config_version_2() -> NonNull<SessionConfig> {
    boxed(SessionConfig::version_2())
}

#[ffi]
pub fn vodozemac_olm_session_config_version(session_config: &SessionConfig) -> u32 {
    session_config.version() as u32
}

#[ffi]
pub fn vodozemac_olm_session_config_free(session_config: NonNull<SessionConfig>) {
    free(session_config)
}
