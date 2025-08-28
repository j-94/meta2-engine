use super::types::Bits;

pub fn trust_from(passed: bool, bits: &Bits) -> f32 {
    if passed && bits.e == 0.0 { 0.9 } else { 0.1 }
}
