use alloc::boxed::Box;
use core::ffi::{c_char, CStr};

// I have absolutely no clue how any of this works or whether it does. Or how even to actually use it from C or C++.
// If you know please test this and let me know how to fix it. (Or least let me know the problem).

// This seems useful: https://ultrasaurus.com/2020/01/writing-c-library-in-rust/

// Idea here is that the caller passes a pointer to out_candidates, where I go and write in the candidates and
// return their length. Then they go and read the values. Then call the below function.
fn shared_ffi_core(
    input: *const c_char,
    out_candidates: *mut *const usize,
    inner: impl FnOnce(&str) -> Box<[usize]>,
) -> usize {
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else { return 0 };

    let result = inner(input);
    let ret = result.len();

    unsafe { out_candidates.write((*result).as_ptr()) };
    core::mem::forget(result);

    ret
}

#[no_mangle]
extern "C" fn find_kashidas_arabic(
    input: *const c_char,
    out_candidates: *mut *const usize,
) -> usize {
    shared_ffi_core(input, out_candidates, super::arabic::find_kashidas)
}

#[no_mangle]
extern "C" fn find_kashidas_syriac(
    input: *const c_char,
    out_candidates: *mut *const usize,
) -> usize {
    shared_ffi_core(input, out_candidates, super::syriac::find_kashidas)
}

#[no_mangle]
extern "C" fn find_kashidas_generic(
    input: *const c_char,
    out_candidates: *mut *const usize,
) -> usize {
    shared_ffi_core(input, out_candidates, super::global::find_kashidas)
}

// This is needed apparently.
#[no_mangle]
extern "C" fn free_candidates(data: *mut usize, len: usize) {
    unsafe {
        drop(Box::from_raw(core::slice::from_raw_parts_mut(data, len)));
    };
}
