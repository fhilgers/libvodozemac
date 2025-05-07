// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::olm::{Account, Session};
use crate::slices::{CErrorStr, CSlice};
use crate::{AsUsize, CResult, Chain, ChainExact, boxed, free};
use jni::JNIEnv;
use jni::objects::JClass;
use macros::ffi;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::{array, str};
use vodozemac::olm::{AccountPickle, Message, PreKeyMessage, SessionConfig, SessionKeys};
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature, olm};

pub fn register_jni(env: &mut JNIEnv, class: &JClass) -> jni::errors::Result<()> {
    env.register_native_methods(
        class,
        &[
            VODOZEMAC_OLM_ACCOUNT_NEW_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_FREE_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_IDENTITY_KEYS_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_ED25519_KEY_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_CURVE25519_KEY_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_SIGN_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_MAX_NUMBER_OF_ONE_TIME_KEYS_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_CREATE_OUTBOUND_SESSION_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_CREATE_INBOUND_SESSION_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_GENERATE_ONE_TIME_KEYS_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_STORED_ONE_TIME_KEY_COUNT_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_ONE_TIME_KEYS_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_GENERATE_FALLBACK_KEY_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_FALLBACK_KEY_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_FORGET_FALLBACK_KEY_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_MARK_KEYS_AS_PUBLISHED_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_PICKLE_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_FROM_PICKLE_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_TO_DEHYDRATED_DEVICE_JNI.into(),
            VODOZEMAC_OLM_ACCOUNT_FROM_DEHYDRATED_DEVICE_JNI.into(),
        ],
    )
}

#[repr(C)]
pub struct IdentityKeys {
    ed25519: NonNull<Ed25519PublicKey>,
    curve25519: NonNull<Curve25519PublicKey>,
}

#[repr(C)]
pub struct InboundCreationResult {
    plaintext: CSlice<u8>,
    session: NonNull<Session>,
}

#[repr(C)]
pub struct OneTimeKeyGenerationResult {
    created: CSlice<NonNull<Curve25519PublicKey>>,
    removed: CSlice<NonNull<Curve25519PublicKey>>,
}

#[repr(C)]
pub struct DehydratedDeviceResult {
    ciphertext: CSlice<u8>,
    nonce: CSlice<u8>,
}

impl From<olm::IdentityKeys> for IdentityKeys {
    fn from(value: olm::IdentityKeys) -> Self {
        Self {
            ed25519: boxed(value.ed25519),
            curve25519: boxed(value.curve25519),
        }
    }
}

impl From<olm::InboundCreationResult> for InboundCreationResult {
    fn from(value: olm::InboundCreationResult) -> Self {
        Self {
            plaintext: value.plaintext.into(),
            session: boxed(RwLock::new(value.session)),
        }
    }
}

impl From<olm::OneTimeKeyGenerationResult> for OneTimeKeyGenerationResult {
    fn from(value: olm::OneTimeKeyGenerationResult) -> Self {
        let created = value.created.into_iter().map(boxed).collect::<Vec<_>>();

        let removed = value.removed.into_iter().map(boxed).collect::<Vec<_>>();

        Self {
            created: created.into(),
            removed: removed.into(),
        }
    }
}

impl AsUsize for IdentityKeys {
    type IntoIter = Chain<array::IntoIter<usize, 1>, array::IntoIter<usize, 1>>;

    fn as_usize(&self) -> Self::IntoIter {
        self.ed25519
            .as_usize()
            .chain_exact(self.curve25519.as_usize())
    }
}

impl AsUsize for InboundCreationResult {
    type IntoIter = Chain<<CSlice<u8> as AsUsize>::IntoIter, array::IntoIter<usize, 1>>;

    fn as_usize(&self) -> Self::IntoIter {
        self.plaintext
            .as_usize()
            .chain_exact(self.session.as_usize())
    }
}

impl AsUsize for OneTimeKeyGenerationResult {
    type IntoIter = Chain<array::IntoIter<usize, 2>, array::IntoIter<usize, 2>>;

    fn as_usize(&self) -> Self::IntoIter {
        self.created.as_usize().chain_exact(self.removed.as_usize())
    }
}

impl AsUsize for DehydratedDeviceResult {
    type IntoIter = Chain<array::IntoIter<usize, 2>, array::IntoIter<usize, 2>>;

    fn as_usize(&self) -> Self::IntoIter {
        self.ciphertext
            .as_usize()
            .chain_exact(self.nonce.as_usize())
    }
}

#[ffi]
pub fn vodozemac_olm_account_new() -> NonNull<Account> {
    boxed(RwLock::new(olm::Account::new()))
}

#[ffi]
pub fn vodozemac_olm_account_free(account: NonNull<Account>) {
    free(account)
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_identity_keys(account: &Account) -> IdentityKeys {
    account.read().identity_keys().into()
}

#[ffi]
pub fn vodozemac_olm_account_ed25519_key(account: &Account) -> NonNull<Ed25519PublicKey> {
    boxed(account.read().ed25519_key())
}

#[ffi]
pub fn vodozemac_olm_account_curve25519_key(account: &Account) -> NonNull<Curve25519PublicKey> {
    boxed(account.read().curve25519_key())
}

#[ffi]
pub fn vodozemac_olm_account_sign(
    account: &Account,
    #[expand] message: &[u8],
) -> NonNull<Ed25519Signature> {
    boxed(account.read().sign(message))
}

#[ffi]
pub fn vodozemac_olm_account_max_number_of_one_time_keys(account: &Account) -> u32 {
    account.read().max_number_of_one_time_keys() as u32
}

#[ffi]
pub fn vodozemac_olm_account_create_outbound_session(
    account: &Account,
    session_config: &SessionConfig,
    identity_key: &Curve25519PublicKey,
    one_time_key: &Curve25519PublicKey,
) -> NonNull<Session> {
    let session =
        account
            .read()
            .create_outbound_session(*session_config, *identity_key, *one_time_key);

    boxed(RwLock::new(session))
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_create_inbound_session(
    account: &Account,
    their_identity_key: &Curve25519PublicKey,
    message: &Message,
    session_keys: &SessionKeys,
) -> CResult<InboundCreationResult, CErrorStr> {
    account
        .write()
        .create_inbound_session(
            *their_identity_key,
            // TODO: ugly, needs changing in upstream
            &PreKeyMessage::wrap(*session_keys, message.clone()),
        )
        .map(Into::into)
        .map_err(Into::into)
        .into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_generate_one_time_keys(
    account: &Account,
    count: u32,
) -> OneTimeKeyGenerationResult {
    account
        .write()
        .generate_one_time_keys(count as usize)
        .into()
}

#[ffi]
pub fn vodozemac_olm_account_stored_one_time_key_count(account: &Account) -> u32 {
    account.read().stored_one_time_key_count() as u32
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_one_time_keys(
    account: &Account,
) -> CSlice<NonNull<Curve25519PublicKey>> {
    let one_time_keys = account.read().one_time_keys();
    let mut pairs = one_time_keys.into_iter().collect::<Vec<_>>();
    pairs.sort_by_key(|(k, _)| *k);

    pairs
        .into_iter()
        .map(|(_, v)| v)
        .map(boxed)
        .collect::<Vec<_>>()
        .into()
}

#[ffi]
pub fn vodozemac_olm_account_generate_fallback_key(
    account: &Account,
) -> Option<NonNull<Curve25519PublicKey>> {
    Some(boxed(account.write().generate_fallback_key()?))
}

#[ffi]
pub fn vodozemac_olm_account_fallback_key(
    account: &Account,
) -> Option<NonNull<Curve25519PublicKey>> {
    Some(boxed(account.read().fallback_key().into_values().next()?))
}

#[ffi]
pub fn vodozemac_olm_account_forget_fallback_key(account: &Account) -> u32 {
    account.write().forget_fallback_key().into()
}

#[ffi]
pub fn vodozemac_olm_account_mark_keys_as_published(account: &Account) {
    account.write().mark_keys_as_published()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_pickle(account: &Account, pickle_key: &[u8; 32]) -> CSlice<u8> {
    account.read().pickle().encrypt(pickle_key).into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_from_pickle(
    #[expand] ciphertext: &[u8],
    pickle_key: &[u8; 32],
) -> CResult<NonNull<Account>, CErrorStr> {
    let ciphertext = str::from_utf8(ciphertext).expect("should be utf8");

    AccountPickle::from_encrypted(ciphertext, pickle_key)
        .map(olm::Account::from_pickle)
        .map(RwLock::new)
        .map(boxed)
        .map_err(Into::into)
        .into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_to_dehydrated_device(
    account: &Account,
    key: &[u8; 32],
) -> CResult<DehydratedDeviceResult, CErrorStr> {
    account
        .read()
        .to_dehydrated_device(key)
        .map(|res| DehydratedDeviceResult {
            ciphertext: res.ciphertext.into(),
            nonce: res.nonce.into(),
        })
        .map_err(Into::into)
        .into()
}

#[ffi]
#[sret]
pub fn vodozemac_olm_account_from_dehydrated_device(
    #[expand] ciphertext: &[u8],
    #[expand] nonce: &[u8],
    key: &[u8; 32],
) -> CResult<NonNull<Account>, CErrorStr> {
    let ciphertext = str::from_utf8(ciphertext).expect("valid utf8");
    let nonce = str::from_utf8(nonce).expect("valid utf8");

    olm::Account::from_dehydrated_device(ciphertext, nonce, key)
        .map(RwLock::new)
        .map(boxed)
        .map_err(Into::into)
        .into()
}
