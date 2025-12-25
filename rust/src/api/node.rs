use flutter_rust_bridge::frb;
use crate::frb_generated::StreamSink;
use libp2p::futures::StreamExt;
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
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig, Event as MdnsEvent},
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
    mdns: Mdns,
}

// 2. MAIN FUNCTION
pub async fn start_p2p_node(sink: StreamSink<String>) {
    // A. Identity & Keys
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    // Send ID to Flutter
    let _ = sink.add(format!("ME:{}", peer_id));

    // B. Setup Kademlia
    let store = MemoryStore::new(peer_id);
    let mut kad_config = KademliaConfig::default();
    kad_config.set_protocol_names(vec![
        libp2p::StreamProtocol::new("/p2pmsg/kad/1.0.0")
    ]);
    let kademlia = Kademlia::with_config(peer_id, store, kad_config);

    // C. Setup Ping
    let ping = Ping::new(PingConfig::new().with_interval(Duration::from_secs(30)));

    // D. Setup mDNS (Local Discovery)
    let mdns = Mdns::new(MdnsConfig::default(), peer_id).expect("Failed to create mDNS behaviour");

    // E. Combine Behaviours
    let behaviour = MyP2PBehaviour { kademlia, ping, mdns };

    // F. Build the Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
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

    // G. START THE EVENT LOOP IN BACKGROUND
    // We listen on all interfaces (0.0.0.0) with a random OS-assigned port (0)
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
    
    // We spawn a Tokio task to keep the node alive.
    // This runs in parallel and won't block the Flutter UI.
    tokio::spawn(async move {
        loop {
            // Wait for the next event from the network
            match swarm.select_next_some().await {
                
                // EVENT: mDNS discovered a new peer!
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        let _ = sink.add(format!("PEER+:{peer_id}"));
                    }
                },

                // EVENT: mDNS lost a peer (expired)
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS peer expired: {peer_id}");
                        let _ = sink.add(format!("PEER-:{peer_id}"));
                    }
                },

                // EVENT: Node started listening
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {address:?}");
                },

                // Ignore other events for now
                _ => {}
            }
        }
    });
}