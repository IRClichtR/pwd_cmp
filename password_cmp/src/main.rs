// The intent of this code is to compare two passwords and check if they're the same
// without using any hashing or encryption. This is a simple comparison function.
// The code is written in Rust and uses the standard library for input/output operations.
// It intend to be resistant to timing attacks by ensuring that the comparison takes the same amount of time regardless of the input.
// to zeroize the memory few options are possible
    // 1. use the `zeroize` crate
    // 2. use `memset` from libc
    // 3. use `std::ptr::write_bytes`

// this include using unsafe code since to zero the memory we need to use `unsafe` block

use std::env;
use zeroize::Zeroize;
// extern crate libc;

pub fn constant_pwd_cmp(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let len = a.len();
    let mut result: u8 = 0;

    unsafe {
        let a_ptr = a.as_ptr();
        let b_ptr = b.as_ptr();
        for i in 0.. len {
            result |= *a_ptr.add(i) ^ *b_ptr.add(i);
        }
    }

    result == 0
}

fn zeroize_memory(pwd: &mut [u8]) {
    pwd.zeroize();
}

// fn zeroize_memory_v2(password: &str) {
//     // zero the memory
//     unsafe {
//         let password_ptr = password.as_ptr();
//         let password_len = password.len();
//         std::ptr::write_bytes(password_ptr as *mut u8, 0, password_len);
//     }
// }

// fn zeroize_memory_v3(pwd: &mut str) {
//     unsafe {
//         libc::memset(pwd.as_ptr() as *mut libc::c_void, 0, pwd.len());
//     }
// }

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage : password_cmp <password> <password>");
        std::process::exit(1);
    }
    let mut password1 = args[1].clone().into_bytes();
    let mut password2 = args[2].clone().into_bytes();

    if constant_pwd_cmp(&password1, &password2) {
        println!("Passwords match");
    } else {
        println!("Passwords do not match");
    }
    // zero the memory
    zeroize_memory(&mut password1);
    zeroize_memory(&mut password2);
}