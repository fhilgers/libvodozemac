// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

fn custom_rng(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let (buf, len) = (buf.as_mut_ptr(), buf.len());
    let res = unsafe { extern_rng(buf, len) };

    if let Ok(err) = std::num::NonZeroU32::try_from(res) {
        Err(getrandom::Error::from(err))
    } else {
        Ok(())
    }
}

getrandom::register_custom_getrandom!(custom_rng);

unsafe extern "C" {
    fn extern_rng(buf: *mut u8, len: usize) -> u32;
}
