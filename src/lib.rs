#![allow(non_snake_case)]

// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use ::macros::ffi;
use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JIntArray, JLongArray, JObject, JShortArray};
use jni::sys::{jbyte, jint, jlong, jshort};
use std::alloc::Layout;
use std::ffi::c_void;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::ptr::{NonNull, slice_from_raw_parts};
use std::{alloc, array, ptr};

pub mod types;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(vm: jni::JavaVM, _: *mut c_void) -> jint {
    fn inner(vm: jni::JavaVM) -> jni::errors::Result<()> {
        let mut env = vm.get_env()?;

        let class_name = |suffix: &str| format!("com/github/fhilgers/vodozemac/bindings/{suffix}");

        let megolm_classes = megolm::MegolmJniClasses {
            group_session: &env.find_class(class_name("megolm/GroupSessionBindingsKt"))?,
            session_config: &env.find_class(class_name("megolm/SessionConfigBindingsKt"))?,
            inbound_group_session: &env
                .find_class(class_name("megolm/InboundGroupSessionBindingsKt"))?,
            message: &env.find_class(class_name("megolm/MessageBindingsKt"))?,
            session_key: &env.find_class(class_name("megolm/SessionKeyBindingsKt"))?,
            exported_session_key: &env
                .find_class(class_name("megolm/ExportedSessionKeyBindingsKt"))?,
        };

        let sas_classes = sas::SasJniClasses {
            established_sas: &env.find_class(class_name("sas/EstablishedSasBindingsKt"))?,
            mac: &env.find_class(class_name("sas/MacBindingsKt"))?,
            sas: &env.find_class(class_name("sas/SasBindingsKt"))?,
            sas_bytes: &env.find_class(class_name("sas/SasBytesBindingsKt"))?,
        };

        let olm_classes = olm::OlmJniClasses {
            account: &env.find_class(class_name("olm/AccountBindingsKt"))?,
            message: &env.find_class(class_name("olm/MessageBindingsKt"))?,
            session: &env.find_class(class_name("olm/SessionBindingsKt"))?,
            session_config: &env.find_class(class_name("olm/SessionConfigBindingsKt"))?,
            session_keys: &env.find_class(class_name("olm/SessionKeysBindingsKt"))?,
        };

        let key_class = &env.find_class(class_name("KeyBindingsKt"))?;

        megolm::register_jni(&mut env, &megolm_classes)?;

        sas::register_jni(&mut env, &sas_classes)?;

        olm::register_jni(&mut env, &olm_classes)?;

        keys::register_jni(&mut env, key_class)?;

        let slice_bindings = env.find_class(class_name("SliceBindingsKt"))?;
        env.register_native_methods(
            &slice_bindings,
            &[
                NativeMethod {
                    name: "alloc",
                    sig: "(II)J",
                    fn_ptr: {
                        fn wrapper(_: JNIEnv, _: JClass, size: jint, align: jint) -> jlong {
                            alloc(size as _, align as _) as _
                        }
                        wrapper as _
                    },
                }
                .into(),
                NativeMethod {
                    name: "dealloc",
                    sig: "(JII)V",
                    fn_ptr: {
                        fn wrapper(_: JNIEnv, _: JClass, ptr: jlong, size: jint, align: jint) {
                            dealloc(ptr as _, size as _, align as _)
                        }
                        wrapper as _
                    },
                }
                .into(),
                NativeMethod {
                    name: "copy_nonoverlapping",
                    sig: "(JLjava/lang/Object;I)V",
                    fn_ptr: {
                        // TODO: maybe other arrays?
                        fn wrapper(
                            mut env: JNIEnv,
                            _: JClass,
                            src: jlong,
                            dest: JObject,
                            size: jint,
                        ) -> jni::errors::Result<()> {
                            if env.is_instance_of(&dest, "[B")? {
                                let src = unsafe {
                                    &*slice_from_raw_parts(src as *mut jbyte, size as usize)
                                };
                                env.set_byte_array_region(JByteArray::from(dest), 0, src)?;
                            } else if env.is_instance_of(&dest, "[S")? {
                                let src = unsafe {
                                    &*slice_from_raw_parts(src as *mut jshort, size as usize / 2)
                                };
                                env.set_short_array_region(JShortArray::from(dest), 0, src)?;
                            } else if env.is_instance_of(&dest, "[I")? {
                                let src = unsafe {
                                    &*slice_from_raw_parts(src as *mut jint, size as usize / 4)
                                };
                                env.set_int_array_region(JIntArray::from(dest), 0, src)?;
                            } else if env.is_instance_of(&dest, "[J")? {
                                let src = unsafe {
                                    &*slice_from_raw_parts(src as *mut jlong, size as usize / 8)
                                };
                                env.set_long_array_region(JLongArray::from(dest), 0, src)?;
                            } else {
                                env.throw_new(
                                    "java/lang/IllegalArgumentException",
                                    "invalid input",
                                )?;
                            }

                            Ok(())
                        }
                        fn outer(
                            env: JNIEnv,
                            class: JClass,
                            src: jlong,
                            dest: JObject,
                            size: jint,
                        ) {
                            let _ = wrapper(env, class, src, dest, size);
                        }
                        outer as _
                    },
                }
                .into(),
            ],
        )?;
        Ok(())
    }

    inner(vm).map(|_| jni::sys::JNI_VERSION_1_6).unwrap_or(-1)
}

#[repr(C, usize)]
#[derive(Clone, Copy, Debug)]
pub enum CResult<T, E> {
    Ok(T),
    Err(E),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZST;

impl From<()> for ZST {
    fn from(_: ()) -> Self {
        ZST
    }
}

impl AsUsize for ZST {
    type IntoIter = array::IntoIter<usize, 0>;

    fn as_usize(&self) -> Self::IntoIter {
        [].into_iter()
    }
}

impl<T, E> From<Result<T, E>> for CResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(ok) => CResult::Ok(ok),
            Err(err) => CResult::Err(err),
        }
    }
}

#[ffi]
pub fn alloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        return NonNull::dangling().as_ptr();
    }

    let layout = Layout::from_size_align(size, align).expect("valid alignment");

    unsafe { alloc::alloc(layout) }
}

#[ffi]
pub fn dealloc(ptr: *mut u8, size: usize, align: usize) {
    if size == 0 {
        return;
    }

    let layout = Layout::from_size_align(size, align).expect("valid alignment");

    unsafe { alloc::dealloc(ptr, layout) }
}

#[ffi]
pub fn copy_nonoverlapping(src: *const c_void, dest: *mut c_void, size: u32) {
    if size == 0 {
        return;
    }

    unsafe { ptr::copy_nonoverlapping(src, dest, size as usize) }
}

pub mod macros;
use crate::slices::{CErrorStr, CSlice};
pub use macros::{boxed, free};

pub mod keys;
pub mod megolm;
pub mod olm;
pub mod sas;
pub mod slices;

pub trait AsUsize {
    type IntoIter: Iterator<Item = usize>;

    fn as_usize(&self) -> Self::IntoIter;
}

impl<T> AsUsize for NonNull<T> {
    type IntoIter = array::IntoIter<usize, 1>;

    fn as_usize(&self) -> Self::IntoIter {
        [self.as_ptr().addr()].into_iter()
    }
}

impl<T> AsUsize for CSlice<T> {
    type IntoIter = array::IntoIter<usize, 2>;

    fn as_usize(&self) -> Self::IntoIter {
        [self.ptr.as_ptr().addr(), self.len].into_iter()
    }
}

impl AsUsize for CErrorStr {
    type IntoIter = array::IntoIter<usize, 2>;

    fn as_usize(&self) -> Self::IntoIter {
        self.0.as_usize()
    }
}
pub enum Either<I1, I2> {
    Left(I1),
    Right(I2),
}

impl<I1, I2, I> Iterator for Either<I1, I2>
where
    I1: Iterator<Item = I>,
    I2: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Either::Left(l) => l.size_hint(),
            Either::Right(r) => r.size_hint(),
        }
    }
}

impl<I1, I2, I> ExactSizeIterator for Either<I1, I2>
where
    I1: ExactSizeIterator<Item = I>,
    I2: ExactSizeIterator<Item = I>,
{
    fn len(&self) -> usize {
        match self {
            Either::Left(l) => l.len(),
            Either::Right(r) => r.len(),
        }
    }
}

pub struct Chain<I1, I2> {
    first: I1,
    second: I2,
}

impl<I1, I2, I> Iterator for Chain<I1, I2>
where
    I1: Iterator<Item = I>,
    I2: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.next().or_else(|| self.second.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l1, h1) = self.first.size_hint();
        let (l2, h2) = self.second.size_hint();

        let h = match (h1, h2) {
            (Some(v1), Some(v2)) => Some(v1 + v2),
            _ => None,
        };

        (l1 + l2, h)
    }
}

impl<I1, I2, I> ExactSizeIterator for Chain<I1, I2>
where
    I1: ExactSizeIterator<Item = I>,
    I2: ExactSizeIterator<Item = I>,
{
    fn len(&self) -> usize {
        self.first.len() + self.second.len()
    }
}

impl<T, E> AsUsize for CResult<T, E>
where
    T: AsUsize,
    E: AsUsize,
{
    type IntoIter = Chain<array::IntoIter<usize, 1>, Either<T::IntoIter, E::IntoIter>>;

    fn as_usize(&self) -> Self::IntoIter {
        match self {
            CResult::Ok(v) => [0].into_iter().chain_exact(Either::Left(v.as_usize())),
            CResult::Err(e) => [1].into_iter().chain_exact(Either::Right(e.as_usize())),
        }
    }
}

/// A macro to transmute between two types without requiring knowing size
/// statically.
macro_rules! transmute {
    ($val:expr) => {
        ::core::mem::transmute_copy(&::core::mem::ManuallyDrop::new($val))
    };
    // This arm is for use in const contexts, where the borrow required to use
    // transmute_copy poses an issue since the compiler hedges that the type
    // being borrowed could have interior mutability.
    ($srcty:ty; $dstty:ty; $val:expr) => {{
        #[repr(C)]
        union Transmute<A, B> {
            src: ::core::mem::ManuallyDrop<A>,
            dst: ::core::mem::ManuallyDrop<B>,
        }
        ::core::mem::ManuallyDrop::into_inner(
            Transmute::<$srcty, $dstty> {
                src: ::core::mem::ManuallyDrop::new($val),
            }
            .dst,
        )
    }};
}

pub trait CollectIntoArray<T: Debug + Default + Copy>: ExactSizeIterator<Item = T> {
    fn collect_into_array<const N: usize>(self) -> [T; N]
    where
        Self: Sized,
    {
        assert!(self.len() <= N);

        let mut result = [Default::default(); N];

        for (index, value) in self.enumerate() {
            *unsafe { result.get_unchecked_mut(index) } = value;
        }

        result
    }
}

pub trait ChainExact: Iterator {
    fn chain_exact<O>(self, other: O) -> Chain<Self, O>
    where
        Self: Sized,
    {
        Chain {
            first: self,
            second: other,
        }
    }
}

impl<T: Debug + Default + Copy, E: ExactSizeIterator<Item = T>> CollectIntoArray<T> for E {}
impl<E: Iterator> ChainExact for E {}

pub fn get_byte_array_region(
    env: &mut JNIEnv,
    src: &JByteArray,
    offset: u32,
    length: u32,
) -> jni::errors::Result<Box<[u8]>> {
    let mut dest = Box::<[u8]>::new_uninit_slice(length as usize);
    env.get_byte_array_region(src, offset as _, unsafe {
        &mut *(dest.as_mut() as *mut _ as *mut _)
    })?;
    Ok(unsafe { dest.assume_init() })
}

pub fn get_byte_array_region_const<const N: usize>(
    env: &mut JNIEnv,
    src: &JByteArray,
    offset: u32,
) -> jni::errors::Result<[u8; N]> {
    let mut dest = [const { MaybeUninit::<u8>::uninit() }; N];
    env.get_byte_array_region(src, offset as _, unsafe {
        &mut *(dest.as_mut() as *mut _ as *mut _)
    })?;
    Ok(unsafe { transmute!(dest) })
}

pub struct NativeMethod {
    name: &'static str,
    sig: &'static str,
    fn_ptr: *mut c_void,
}

impl From<NativeMethod> for jni::NativeMethod {
    fn from(value: NativeMethod) -> Self {
        jni::NativeMethod {
            name: value.name.into(),
            sig: value.sig.into(),
            fn_ptr: value.fn_ptr,
        }
    }
}
