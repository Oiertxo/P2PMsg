use libp2p::identity;
use std::fs;
use std::path::PathBuf;

pub fn get_or_create_identity(storage_path: &str, instance_name: &str) -> identity::Keypair {
    let mut path = PathBuf::from(storage_path);
    path.push(format!("identity_{}.bin", instance_name));

    if path.exists() {
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(keypair) = identity::Keypair::from_protobuf_encoding(&bytes) {
                return keypair;
            }
        }
    }

    let keypair = identity::Keypair::generate_ed25519();
    if let Ok(bytes) = keypair.to_protobuf_encoding() {
        let _ = fs::write(&path, bytes);
    }
    keypair
}