// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::ptr::NonNull;

#[inline(always)]
pub fn boxed<T>(value: T) -> NonNull<T> {
    let boxed = Box::new(value);
    let raw = Box::into_raw(boxed);
    unsafe { NonNull::new_unchecked(raw) }
}

#[inline(always)]
pub fn free<T>(value: NonNull<T>) {
    let raw = NonNull::as_ptr(value);
    let boxed = unsafe { Box::from_raw(raw) };
    drop(boxed);
}

#[macro_export]
macro_rules! define_and_assert_ptr_sized {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(C)]
        $vis struct $name {
            $( $field: $ty ),*
        }

        const _: () = {
            use std::mem::size_of;
            const PTR_SIZE: usize = size_of::<*const ()>();
            $(
                const _: () = assert!(
                    size_of::<$ty>() == PTR_SIZE,
                    concat!(
                        "Field `",
                        stringify!($field),
                        "` is not pointer-sized"
                    )
                );
            )*
        };
    };
}
