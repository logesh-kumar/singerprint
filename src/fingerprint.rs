use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AudioFingerprint {
    pub peaks: Vec<(f32, f32)>,  // (frequency, time) pairs
    pub hash: Vec<u64>,          // Fingerprint hash
}