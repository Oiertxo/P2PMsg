use std::time::Duration;
use flutter_rust_bridge::frb;
use crate::frb_generated::StreamSink;
use futures::StreamExt;
use libp2p::{
    gossipsub,
    kad::{
        store::MemoryStore,
        Behaviour as Kademlia, 
        Config as KademliaConfig,
    },
    ping::{Behaviour as Ping, Config as PingConfig},
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig, Event as MdnsEvent},
    swarm::SwarmEvent,
    SwarmBuilder, PeerId,
    identify,
    relay,
    dcutr,
    multiaddr::Protocol,
};
use tokio::sync::mpsc;

// Import custom modules
use crate::identity::get_or_create_identity;
use crate::behaviour::{MyP2PBehaviour, MyP2PBehaviourEvent};
use crate::transport::build_transport;
pub use crate::config::AppConfig;

// Sends info to Flutter
static COMMAND_SENDER: std::sync::OnceLock<mpsc::UnboundedSender<(String, String)>> = std::sync::OnceLock::new();

#[frb(sync)]
pub fn send_message(recipient: String, msg: String) {
    if let Some(sender) = COMMAND_SENDER.get() {
        let _ = sender.send((recipient, msg));
    }
}

pub async fn start_p2p_node(
    sink: StreamSink<String>,
    storage_path: String,
    instance_name: String,
    config: AppConfig,
) {
    // Identity and keys
    let id_keys = get_or_create_identity(&storage_path, &instance_name);
    let peer_id = PeerId::from(id_keys.public());
    let _ = sink.add(format!("ME:{}", peer_id));

    // Transport
    let (transport, relay_client) = build_transport(&id_keys, peer_id);

    // Behaviours
    let mut kad_config = KademliaConfig::default();
    kad_config.set_protocol_names(vec![libp2p::StreamProtocol::new("/p2p_msg/kad/1.0.0")]);
    
    let behaviour = MyP2PBehaviour {
        kademlia: Kademlia::with_config(peer_id, MemoryStore::new(peer_id), kad_config),
        ping: Ping::new(PingConfig::new().with_interval(Duration::from_secs(30))),
        mdns: Mdns::new(MdnsConfig::default(), peer_id).expect("mDNS error"),
        gossipsub: gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .build().unwrap(),
        ).expect("Gossipsub error"),
        identify: identify::Behaviour::new(identify::Config::new(
            "/p2p_msg/id/1.0.0".to_string(), 
            id_keys.public()
        )),
        relay_client,
        relay_server: relay::Behaviour::new(peer_id, relay::Config::default()),
        dcutr: dcutr::Behaviour::new(peer_id),
    };

    // Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_other_transport(|_| transport).expect("Transport failed")
        .with_behaviour(|_| behaviour).expect("Behaviour failed")
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

    // Config listening
    let listen_addr_str = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port);
    let listen_addr = listen_addr_str.parse::<libp2p::Multiaddr>().unwrap();

    if config.is_bootstrap_node {
        swarm.listen_on(listen_addr).expect("Listen failed");
        println!("VPS Mode: Port {}", config.listen_port);
    } else {
        swarm.listen_on(listen_addr).unwrap();
        if !config.relay_address.is_empty() {
            if let Ok(addr) = config.relay_address.parse::<libp2p::Multiaddr>() {
                swarm.listen_on(addr).expect("Relay listen failed");
            }
        }
        for addr_str in config.bootstrap_nodes {
            if let Ok(mut addr) = addr_str.parse::<libp2p::Multiaddr>() {
                if let Some(Protocol::P2p(remote_peer_id)) = addr.pop() {
                    swarm.behaviour_mut().kademlia.add_address(&remote_peer_id, addr.clone());
                    let _ = swarm.dial(addr.with(Protocol::P2p(remote_peer_id)));
                }
            }
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<(String, String)>();
    let _ = COMMAND_SENDER.set(tx);

    // Event loop
    loop {
        tokio::select! {
            // Command from Flutter
            Some((recipient, msg_to_send)) = rx.recv() => {
                if recipient == "REFRESH" {
                    println!("Refreshing node discovery...");
                    // Refresh network
                    let _ = swarm.behaviour_mut().kademlia.bootstrap();
                    // Refresh mDNS
                    let random_peer = PeerId::random();
                    swarm.behaviour_mut().kademlia.get_closest_peers(random_peer);
                    for peer_id in swarm.connected_peers() {
                        let _ = sink.add(format!("PEER+:{}", peer_id));
                    }
                } else {
                    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
                    // Publish message
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg_to_send.as_bytes()) {
                        println!("Publish error: {e:?}");
                    } else {
                        // Send ACK to Flutter
                        let _ = sink.add(format!("MSG_SENT:{}:{}", recipient, msg_to_send)); 
                    }
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

                // Peer discovered (mDNS)
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                        let _ = sink.add(format!("PEER+:{peer_id}"));
                        println!("Connection opened with {peer_id} using mDNS");
                    }
                },

                // Peer discovered (Kademlia)
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Kademlia(libp2p::kad::Event::RoutingUpdated { peer, .. })) => {
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                    let _ = sink.add(format!("PEER+:{peer}"));
                    println!("Connection opened with {peer} using Kademlia");
                },

                // Any connection
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    let _ = sink.add(format!("PEER+:{peer_id}"));
                    println!("Connection opened with {peer_id} using any method");
                },

                // Peer disconnected
                SwarmEvent::ConnectionClosed { peer_id, num_established, .. } => {
                    if num_established == 0 {
                        println!("Connection closed with {peer_id}");
                        // Update Flutter
                        let _ = sink.add(format!("PEER-:{peer_id}"));
                        
                        // Clear Gossipsub/Kademlia
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                
                // Peer expired
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
                        println!("mDNS expired for {}: address {} removed form routing table", peer_id, multiaddr);
                    }
                },

                // Info of Peer
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                    println!("Identify: info of Peer {peer_id}");
                    for addr in info.listen_addrs {
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                    }
                },

                _ => {}
            }
        }
    }
}

#[frb(sync)]
pub fn refresh_node() {
    if let Some(sender) = COMMAND_SENDER.get() {
        let _ = sender.send(("REFRESH".to_string(), "REFRESH".to_string()));
    }
}