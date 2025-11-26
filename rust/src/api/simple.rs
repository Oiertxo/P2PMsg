use flutter_rust_bridge::frb;
use libp2p::{PeerId, identity};

#[frb(sync)]
pub fn generate_my_identity() -> String {
    // Private key
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    
    format!("Hi! My PeerID is: {}", peer_id)
}