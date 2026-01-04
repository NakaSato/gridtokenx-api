use solana_sdk::signature::{Keypair, Signer};

fn main() {
    let kp = Keypair::new();
    let pubkey_orig = kp.pubkey();
    let bytes = kp.to_bytes();
    
    println!("Original Pubkey: {}", pubkey_orig);
    println!("Total Bytes: {}", bytes.len());
    
    // Attempt 1: From first 32 bytes (seed)
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes[0..32]);
    let kp_der = Keypair::new_from_array(seed);
    println!("Derived via seed: {}", kp_der.pubkey());
    
    // Attempt 2: Full bytes (64)
    // Solana SDK Keypair doesn't have from_bytes(64)?
    // Let's check if it matches.
    
    if pubkey_orig == kp_der.pubkey() {
        println!("✅ MATCH: First 32 bytes are the seed.");
    } else {
        println!("❌ MISMATCH: First 32 bytes are NOT the seed!");
    }
}
