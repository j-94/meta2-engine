---
lane: D
title: Ask–Act Gate Function (Rust)
source: src/engine/kernel.rs
tags: [code, rust, kernel, safety]
bits: {A: 1, U: 0, P: 1, E: 0, Δ: 0, I: 0, R: 0, T: 1}
---

snippet (rust)
```rust
pub fn enforce_ask_act_gate(bits: &ExtendedBits) -> Result<(), String> {
    if !(bits.a >= 1.0 && bits.p >= 1.0 && bits.d == 0.0) {
        return Err(format!(
            "Ask-Act gate failed: A={:.1}, P={:.1}, Δ={:.1}", bits.a, bits.p, bits.d
        ));
    }
    Ok(())
}
```

example
- Call before any side-effecting action; on Err, return a clarifying payload.

CTA
- Port this function to your loop’s language and wire it at boundaries.

