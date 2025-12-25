use flutter_rust_bridge::frb;
use crate::frb_generated::StreamSink;
use libp2p::futures::StreamExt;
use libp2p::{
    identity,
    gossipsub,
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
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Sends messsages to Swarm
static COMMAND_SENDER: std::sync::OnceLock<mpsc::UnboundedSender<(String, String)>> = std::sync::OnceLock::new();

// NETWORK BEHAVIOUR
#[derive(NetworkBehaviour)]
struct MyP2PBehaviour {
    kademlia: Kademlia<MemoryStore>,
    ping: Ping,
    mdns: Mdns,
    gossipsub: gossipsub::Behaviour,
}

#[frb(sync)]
pub fn send_message(recipient: String, msg: String) {
    if let Some(sender) = COMMAND_SENDER.get() {
        let _ = sender.send((recipient, msg));
    } else {
        println!("Error: Node not ready.");
    }
}

// MAIN FUNCTION
pub async fn start_p2p_node(sink: StreamSink<String>) {
    // Identity & Keys
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    // Send ID to Flutter
    let _ = sink.add(format!("ME:{}", peer_id));

    // Setup Kademlia
    let store = MemoryStore::new(peer_id);
    let mut kad_config = KademliaConfig::default();
    kad_config.set_protocol_names(vec![
        libp2p::StreamProtocol::new("/p2pmsg/kad/1.0.0")
    ]);
    let kademlia = Kademlia::with_config(peer_id, store, kad_config);

    // Setup Ping
    let ping = Ping::new(PingConfig::new().with_interval(Duration::from_secs(30)));

    // Setup mDNS (Local Discovery)
    let mdns = Mdns::new(MdnsConfig::default(), peer_id).expect("Failed to create mDNS behaviour");

    // Setup Gossipsub
    let gossip_msg_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Valid config");

    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
        gossip_msg_config,
    ).expect("Correct behaviour");

    // Subscribe to global channel
    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
    gossipsub.subscribe(&topic).unwrap();

    // Combine Behaviours
    let behaviour = MyP2PBehaviour { kademlia, ping, mdns, gossipsub };

    // Build the Swarm
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
    
    // We listen on all interfaces (0.0.0.0) with a random OS-assigned port (0)
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

    // Create command channel
    let (tx, mut rx) = mpsc::unbounded_channel::<(String, String)>();
    let _ = COMMAND_SENDER.set(tx);
    
    // Tokio task to keep the node alive.
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Command from Flutter
                Some((recipient, msg_to_send)) = rx.recv() => {
                    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
                    // Publish message
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg_to_send.as_bytes()) {
                        println!("Publish error: {e:?}");
                    } else {
                        // Send ACK to Flutter
                        let _ = sink.add(format!("MSG_SENT:{}:{}", recipient, msg_to_send)); 
                    }
                }

                // Network events
                event = swarm.select_next_some() => match event {                    
                    // Receive message from Peer
                    SwarmEvent::Behaviour(MyP2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: _,
                        message,
                    })) => {
                        let text = String::from_utf8_lossy(&message.data);
                        println!("Message received from {peer_id}: {text}");
                        let _ = sink.add(format!("MSG:{}:{}", peer_id, text));
                    },

                    // Peer discovered
                    SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, _multiaddr);
                            let _ = sink.add(format!("PEER+:{peer_id}"));
                        }
                    },
                    
                    // Peer disconnected
                    SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                         for (peer_id, _multiaddr) in list {
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            swarm.behaviour_mut().kademlia.remove_address(&peer_id, &_multiaddr);
                            let _ = sink.add(format!("PEER-:{peer_id}"));
                        }
                    },
                    _ => {}
                }
            }
        }
    });
}