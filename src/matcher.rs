use std::collections::HashMap;
use crate::fingerprint::AudioFingerprint;

pub struct FingerprintMatcher {
    database: HashMap<String, AudioFingerprint>,
}

impl FingerprintMatcher {
    pub fn new() -> Self {
        FingerprintMatcher {
            database: HashMap::new(),
        }
    }
    
    pub fn add_fingerprint(&mut self, name: &str, fingerprint: AudioFingerprint) {
        self.database.insert(name.to_string(), fingerprint);
    }
    
    pub fn find_match(&self, fingerprint: &AudioFingerprint) -> Option<String> {
        let mut best_match = None;
        let mut best_score = 0;
        
        for (name, stored) in &self.database {
            let score = self.compare_fingerprints(fingerprint, stored);
            if score > best_score {
                best_score = score;
                best_match = Some(name.clone());
            }
        }
        
        if best_score > 10 {  // Adjust threshold as needed
            best_match
        } else {
            None
        }
    }
    
    pub fn compare_fingerprints(&self, fp1: &AudioFingerprint, fp2: &AudioFingerprint) -> u32 {
        let mut score = 0;
        
        for hash1 in &fp1.hash {
            if fp2.hash.contains(hash1) {
                score += 1;
            }
        }
        
        score
    }
}