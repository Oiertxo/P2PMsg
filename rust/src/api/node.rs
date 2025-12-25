use flutter_rust_bridge::frb;
use libp2p::{
    identity,
    kad::{
        store::MemoryStore,
        // UPDATE: 'Kademlia' is now 'Behaviour' and 'KademliaConfig' is 'Config'
        // We use 'as' to rename them locally so the rest of the code makes sense.
        Behaviour as Kademlia, 
        Config as KademliaConfig,
    },
    ping::{Behaviour as Ping, Config as PingConfig},
    swarm::{NetworkBehaviour, SwarmEvent},
    SwarmBuilder, PeerId,
};
use std::time::Duration;

// 1. NETWORK BEHAVIOUR
#[derive(NetworkBehaviour)]
struct MyP2PBehaviour {
    // The struct is technically 'Behaviour', but we aliased it to 'Kademlia' above
    kademlia: Kademlia<MemoryStore>,
    ping: Ping,
}

// 2. MAIN FUNCTION
#[frb(sync)]
pub fn start_p2p_node() -> String {
    // A. Identity & Keys
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    // B. Setup Kademlia
    let store = MemoryStore::new(peer_id);
    // 'KademliaConfig' is actually 'Config' aliased
    let mut kad_config = KademliaConfig::default();
    
    kad_config.set_protocol_names(vec![
        libp2p::StreamProtocol::new("/p2pmsg/kad/1.0.0")
    ]);
    
    // We use the aliased name 'Kademlia' (which is 'Behaviour')
    let kademlia = Kademlia::with_config(peer_id, store, kad_config);

    // C. Setup Ping
    let ping = Ping::new(PingConfig::new().with_interval(Duration::from_secs(30)));

    // D. Combine Behaviours
    let behaviour = MyP2PBehaviour { kademlia, ping };

    // E. Build the Swarm
    let _swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio() 
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .expect("Failed to build TCP transport")
        .with_behaviour(|_| behaviour)
        .expect("Failed to build behaviour")
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // F. Return the ID
    peer_id.to_string()
}