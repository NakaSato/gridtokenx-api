use solana_sdk::signature::{Keypair, Signer};
fn main() {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    let bytes = keypair.to_bytes();
    println!("Original Pubkey: {}", pubkey);
    let secret = &bytes[0..32];
    let reconstructed = Keypair::new_from_array(secret.try_into().unwrap());
    println!("Reconstructed Pubkey: {}", reconstructed.pubkey());
    assert_eq!(pubkey, reconstructed.pubkey());
    println!("SUCCESS");
}
