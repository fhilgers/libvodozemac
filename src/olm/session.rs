// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::olm::Session;
use crate::slices::{CErrorStr, CSlice};
use crate::{AsUsize, CResult, Either, boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::{array, str};
use vodozemac::olm;
use vodozemac::olm::{Message, PreKeyMessage, SessionConfig, SessionKeys, SessionPickle};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_OLM_SESSION_FREE_JNI.into(),
            VODOZEMAC_OLM_SESSION_SESSION_ID_JNI.into(),
            VODOZEMAC_OLM_SESSION_HAS_RECEIVED_MESSAGE_JNI.into(),
            VODOZEMAC_OLM_SESSION_ENCRYPT_JNI.into(),
            VODOZEMAC_OLM_SESSION_SESSION_KEYS_JNI.into(),
            VODOZEMAC_OLM_SESSION_SESSION_CONFIG_JNI.into(),
            VODOZEMAC_OLM_SESSION_DECRYPT_JNI.into(),
            VODOZEMAC_OLM_SESSION_PICKLE_JNI.into(),
            VODOZEMAC_OLM_SESSION_FROM_PICKLE_JNI.into(),
        ],
    )
}

#[repr(C, usize)]
#[derive(Clone, Copy)]
pub enum OlmMessage<'a> {
    Normal {
        message: &'a Message,
    },
    PreKey {
        message: &'a Message,
        session_keys: &'a SessionKeys,
    },
}

impl<'a> OlmMessage<'a> {
    pub fn new(message: &'a Message, session_keys: Option<&'a SessionKeys>) -> Self {
        match session_keys {
            Some(session_keys) => OlmMessage::PreKey {
                message,
                session_keys,
            },
            None => OlmMessage::Normal { message },
        }
    }
}

impl AsUsize for OlmMessage<'_> {
    type IntoIter = Either<array::IntoIter<usize, 2>, array::IntoIter<usize, 3>>;

    fn as_usize(&self) -> Self::IntoIter {
        match *self {
            OlmMessage::Normal { message } => {
                let message = (&raw const *message).addr();
                Either::Left([0, message].into_iter())
            }
            OlmMessage::PreKey {
                message,
                session_keys,
            } => {
                let message = (&raw const *message).addr();
                let session_keys = (&raw const *session_keys).addr();
                Either::Right([1, message, session_keys].into_iter())
            }
        }
    }
}

impl From<olm::OlmMessage> for OlmMessage<'static> {
    fn from(value: olm::OlmMessage) -> Self {
        match value {
            olm::OlmMessage::Normal(normal) => OlmMessage::Normal {
                message: Box::leak(Box::new(normal)),
            },
            olm::OlmMessage::PreKey(pre_key) => OlmMessage::PreKey {
                message: Box::leak(Box::new(pre_key.message().clone())),
                session_keys: Box::leak(Box::new(pre_key.session_keys())),
            },
        }
    }
}

impl From<OlmMessage<'_>> for olm::OlmMessage {
    fn from(value: OlmMessage) -> Self {
        match value {
            OlmMessage::PreKey {
                message,
                session_keys,
            } => olm::OlmMessage::PreKey(PreKeyMessage::wrap(*session_keys, message.clone())),
            OlmMessage::Normal { message } => olm::OlmMessage::Normal(message.clone()),
        }
    }
}

#[ffi]
pub fn vodozemac_olm_session_free(session: NonNull<Session>) {
    free(session)
}

#[ffi]
#[sret]
pub fn vodozemac_olm_session_session_id(session: &Session) -> CSlice<u8> {
    session.read().session_id().into()
}

#[ffi]
pub fn vodozemac_olm_session_has_received_message(session: &Session) -> u32 {
    session.read().has_received_message().into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_session_encrypt(
    session: &Session,
    #[expand] plaintext: &[u8],
) -> OlmMessage<'static> {
    session.write().encrypt(plaintext).into()
}

#[ffi]
pub fn vodozemac_olm_session_session_keys(session: &Session) -> NonNull<SessionKeys> {
    boxed(session.read().session_keys())
}

#[ffi]
pub fn vodozemac_olm_session_session_config(session: &Session) -> NonNull<SessionConfig> {
    boxed(session.read().session_config())
}

#[ffi]
#[sret]
pub fn vodozemac_olm_session_decrypt(
    session: &Session,
    message: &Message,
    session_keys: Option<&SessionKeys>,
) -> CResult<CSlice<u8>, CErrorStr> {
    // TODO: zero copy?
    let message = message.clone();

    let olm_message = match session_keys {
        Some(keys) => olm::OlmMessage::PreKey(PreKeyMessage::wrap(*keys, message)),
        None => olm::OlmMessage::Normal(message),
    };

    session
        .write()
        .decrypt(&olm_message)
        .map(Into::into)
        .map_err(Into::into)
        .into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_session_pickle(session: &Session, pickle_key: &[u8; 32]) -> CSlice<u8> {
    session.read().pickle().encrypt(pickle_key).into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_session_from_pickle(
    #[expand] ciphertext: &[u8],
    pickle_key: &[u8; 32],
) -> CResult<NonNull<Session>, CErrorStr> {
    let ciphertext = str::from_utf8(ciphertext).expect("should be utf8");

    SessionPickle::from_encrypted(ciphertext, pickle_key)
        .map(olm::Session::from_pickle)
        .map(RwLock::new)
        .map(boxed)
        .map_err(Into::into)
        .into()
}
