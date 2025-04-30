// The intent of this code is to compare two passwords and check if they're the same
// without using any hashing or encryption. This is a simple comparison function.
// The code is written in Rust and uses the standard library for input/output operations.
// It intends to be resistant to timing attacks by ensuring that the comparison takes 
// the same amount of time regardless of the input.

use std::env;
use zeroize::Zeroize;
// extern crate libc;

#[inline(never)]
pub fn constant_pwd_cmp(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let len = a.len();
    let mut result: u8 = 0;

    //the unsafe block here is necessary to access the raw pointers of the byte slices
    unsafe {
        let a_ptr = a.as_ptr();
        let b_ptr = b.as_ptr();

        for i in 0.. len {
            // read the bytes from the pointers
            // this is a constant time operation
            // by using read_volatile
            // this ensures that the compiler does not optimize away the reads
            // and non-zero if they are not 
            let a_val = std::ptr::read_volatile(a_ptr.add(i));
            let b_val = std::ptr::read_volatile(b_ptr.add(i));
            // XOR the bytes and accumulate the result
            // this is a constant time operation 
            // because it does not depend on the values of the bytes
            // the result will be 0 if all bytes are equal
            // and non-zero if they are not
            result |= a_val ^ b_val;
        }
    }
    // check if the result is zero
    let mut is_equal : u8= 0;

    // Use unsafe block to read the result
    // this is necessary to prevent the compiler from optimizing away the read
    // and to ensure that the read is volatile
    unsafe {
        is_equal = std::ptr::read_volatile(&(result == 0) as *const bool as *const u8);
    }
    // return true if the result is zero
    // and false if it is not
    is_equal != 0
}

fn zeroize_memory(pwd: &mut [u8]) {
    pwd.zeroize();
}

// vulnerable impl for testing

fn vulnerable_pwd_cmp(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let len = a.len();
    let mut result = true;

    // the unsafe block here is necessary to access the raw pointers of the byte slices
    unsafe {
        let a_ptr = a.as_ptr();
        let b_ptr = b.as_ptr();
        for i in 0.. len {
            // this is not a constant time operation
            // because it depends on the values of the bytes
            // if the bytes are equal, the loop will continue
            // if they are not, the loop will break
            result &= *a_ptr.add(i) == *b_ptr.add(i);
        }
    }

    result
}

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage : password_cmp <password> <password>");
        std::process::exit(1);
    }
    // get the passwords from the command line arguments
    // and convert them to byte arrays
    // this is necessary because the constant time comparison function
    // works with byte arrays
    // and not with strings
    let mut password1 = args[1].clone().into_bytes();
    let mut password2 = args[2].clone().into_bytes();

    if constant_pwd_cmp(&password1, &password2) {
        println!("Passwords match");
    } else {
        println!("Passwords do not match");
    }
    // zero the memory
    // this is necessary to prevent the passwords from being leaked
    // in memory
    // this is a good practice to follow
    // when dealing with sensitive information
    zeroize_memory(&mut password1);
    zeroize_memory(&mut password2);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};
    use rand::Rng;

    fn random_string_gen(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        
        let random_string = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        println!("=== Random strig ==  {} ===", random_string);
        random_string
    }

    #[test]
    fn test_matchig_passwords() {
        let password = b"secure_password123";
        assert!(constant_pwd_cmp(password, password));
    }

    #[test]
    fn test_different_content_same_pwd() {
        let password1 = b"secure_password123";
        let password2 = b"secuer_pas^7word123";
        assert!(!constant_pwd_cmp(password1, password2));
    }

    #[test]
    fn test_different_length() {
        let password1 = b"secure_password123";
        let password2 = b"secure_password1234";
        assert!(!constant_pwd_cmp(password1, password2));
    }

    #[test]
    fn test_empty_passwords() {
        let password1 = b"";
        let password2 = b"";
        assert!(constant_pwd_cmp(password1, password2));
    }

    #[test]
    fn test_timing_constitency() {
        let pwd = random_string_gen(32);
        let pwd_bytes = pwd.as_bytes();

        let iterations = 1000;
        // let positions_to_test = [0, 5, 10, 15, 20, 25, 30, 31];
        let positions_to_test = [31, 30, 25, 20, 15, 10, 5, 0];

        let mut timings: Vec<Duration> = Vec::new();

        for &position in &positions_to_test {
            let mut test_password = pwd.clone();
            let char_to_change = (test_password.as_bytes()[position] as char).to_string();
            let replace_char = if char_to_change == "A" { "B" } else { "A" };
            test_password.replace_range(position..position+1, replace_char);
            let test_bytes = test_password.as_bytes();

            // mesure exec time
            let mut total_duration = Duration::new(0, 0);
            for _ in 0..iterations {
                let start = Instant::now();
                let _ = constant_pwd_cmp(pwd_bytes, test_bytes);
                total_duration += start.elapsed();
                std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
                // println!("===Duration = {:?}===", start.elapsed())
            }
            println!("=== total_duration = {:?} ===", total_duration);
            let avg_duration = total_duration / iterations as u32;
            println!("=== Avg duration for position {}: {:?} ===", position, avg_duration);
            timings.push(avg_duration);            
        }

        let avg_timing = timings.iter()
            .copied()
            .sum::<Duration>() / timings.len() as u32;

        println!("=== Average timing: {:?} ===", avg_timing);
        let variance: f64 = timings.iter()
            .map(|duration| {
        let diff = if *duration > avg_timing {
            duration.as_nanos() as f64 - avg_timing.as_nanos() as f64
            } else {
                avg_timing.as_nanos() as f64 - duration.as_nanos() as f64
            };
            diff * diff
        })
            .sum::<f64>() / timings.len() as f64;

        let std_dev = Duration::from_nanos((variance.sqrt()) as u64);

        let rel_std_dev = (std_dev.as_nanos() as f64 / avg_timing.as_nanos() as f64) * 100.0;
        assert!(rel_std_dev < 10.0,
            "Timing variation too high: {}% (expected < 10%)", rel_std_dev);
    }
    

}