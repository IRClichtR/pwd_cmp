# pwd_cmp
This project explores the implementation of a secure password comparison function in Rust using unsafe code. It demonstrates how to create a constant-time comparison to prevent timing attacks and provides insights into testing for timing vulnerabilities.
As a Rust learner, I welcome any contributions that could improve this implementation and help me understand better practices.This little exercise is meant to get hands on unsafe territory in rust by using unsafe code to ensure password comparison function in Rust.

It gives also a gasp on testing.

By the time this readme has been donne I'm still a learner. I'm very opened to any contribution that could improve this implementation and help me understand what i should do better.

## First att:
```rust
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
            // XOR the bytes and accumulate the result
            // this is a constant time operation 
            // because it does not depend on the values of the bytes
            // the result will be 0 if all bytes are equal
            // and non-zero if they are not
            result |= *a_ptr.add(i) ^ *b_ptr.add(i);
        }
    }

    result == 0
}
```
This solution seems theoretically sound, but when measuring timing variance across 10,000 attempts, the results revealed unexpected behavior:
```bash
---- tests::test_timing_constitency stdout ----
=== Random strig ==  WJ6bsGCeUAOTP8IUrddYkJvPTBYfYUJS ===
=== total_duration = 500.712µs ===
=== Avg duration for position 0: 500ns ===
=== total_duration = 507.279µs ===
=== Avg duration for position 5: 507ns ===
=== total_duration = 359.831µs ===
=== Avg duration for position 10: 359ns ===
=== total_duration = 196.002µs ===
=== Avg duration for position 15: 196ns ===
=== total_duration = 195.936µs ===
=== Avg duration for position 20: 195ns ===
=== total_duration = 197.622µs ===
=== Avg duration for position 25: 197ns ===
=== total_duration = 197.68µs ===
=== Avg duration for position 30: 197ns ===
=== total_duration = 201.414µs ===
=== Avg duration for position 31: 201ns ===
=== Average timing: 294ns ===

thread 'tests::test_timing_constitency' panicked at src/main.rs:193:9:
Timing variation too high: 44.5578231292517% (expected < 10%)
```
Why is it so:
For the sake of testing we used an only string and tested many positions in the string. The pattern is a decreasing duration. That strongly suggests that cache optimization caused the most of the problem. It is not a usage we're expecting on a password attempt, but in a context of trying to make this function timing-attack safe, we cannot accept that result.

## Understanding the Timing Discrepancies

### - CPU cache Effects
For testing purposes, we used a single string and tested various positions within it. The pattern shows decreasing execution times, strongly suggesting CPU cache effects. Although this specific scenario might not represent typical password comparison usage, it's essential to address for robust protection against timing attacks.
Modern CPUs use multiple levels of cache (L1, L2, L3) to speed up memory access. The first access to memory locations will be slower than subsequent accesses to nearby memory.
To confirm this was the issue, we reversed the order of positions tested:
```bash
---- tests::test_timing_constitency stdout ----
=== Random strig ==  5bSsMbmCgijyfHZ2M86krjliZzHXgyxZ ===
=== total_duration = 507.959µs ===
=== Avg duration for position 31: 507ns ===
=== total_duration = 499.511µs ===
=== Avg duration for position 30: 499ns ===
=== total_duration = 224.262µs ===
=== Avg duration for position 25: 224ns ===
=== total_duration = 199.576µs ===
=== Avg duration for position 20: 199ns ===
=== total_duration = 199.584µs ===
=== Avg duration for position 15: 199ns ===
=== total_duration = 199.549µs ===
=== Avg duration for position 10: 199ns ===
=== total_duration = 202.601µs ===
=== Avg duration for position 5: 202ns ===
=== total_duration = 201.665µs ===
=== Avg duration for position 0: 201ns ===
=== Average timing: 278ns ===
```
As expected, position 31 now showed the longest execution time, confirming our cache hypothesis.

## Other potential causes:

### 1. Compiler optimizations
Modern compilers apply sophisticated optimizations that can alter timing characteristics even when attempting to write constant-time code. These can be identified by inconsistent timings or differences in behavior between production and debug modes.

### 2. Memory access pattern and Alignments
Memory alignment and access patterns can cause timing variations due to how CPUs fetch and process memory.

### 3. Branch prediction and Instruction Pipelining
CPUs use branch prediction to speculatively execute instructions. When a prediction is wrong, the pipeline must be flushed, causing timing variations.

## Code optimization results
After implementing improvements to mitigate cache effects, we observed:
```bash
---- tests::test_timing_constitency stdout ----
=== Random strig ==  YvfdZ83bmxGD8JP8kWoLuGGbwXpSWMO5 ===
=== total_duration = 354.642µs ===
=== Avg duration for position 31: 354ns ===
=== total_duration = 795.698µs ===
=== Avg duration for position 30: 795ns ===
=== total_duration = 486.937µs ===
=== Avg duration for position 25: 486ns ===
=== total_duration = 315.994µs ===
=== Avg duration for position 20: 315ns ===
=== total_duration = 319.279µs ===
=== Avg duration for position 15: 319ns ===
=== total_duration = 318.82µs ===
=== Avg duration for position 10: 318ns ===
=== total_duration = 312.554µs ===
=== Avg duration for position 5: 312ns ===
=== total_duration = 322.363µs ===
=== Avg duration for position 0: 322ns ===
=== Average timing: 402ns ===
```
Our optimizations improved cache sensitivity, but even with a warm cache, there remains more variance than ideal for perfect security. This highlights the challenges of implementing truly constant-time operations in high-level languages.