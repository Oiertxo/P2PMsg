use std::time::{Duration, Instant};
use std::collections::HashMap;
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
use p2p_core::identity::get_or_create_identity;
use p2p_core::behaviour::{MyP2PBehaviour, MyP2PBehaviourEvent};
use p2p_core::transport::build_transport;
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
    let listen_addr = "/ip4/0.0.0.0/tcp/0".parse::<libp2p::Multiaddr>().unwrap();
    swarm.listen_on(listen_addr).expect("Failed to listen on random port");
    let mut relay_peer_id: Option<PeerId> = None;
    let mut relay_address_to_dial: Option<libp2p::Multiaddr> = None;

    // Connect to relay
    if !config.relay_address.is_empty() {
        if let Ok(relay_addr) = config.relay_address.parse::<libp2p::Multiaddr>() {
            println!("Trying to listen via Relay: {:?}", relay_addr);
            relay_address_to_dial = Some(relay_addr.clone());

            relay_peer_id = relay_addr.iter().find_map(|p| match p {
                Protocol::P2p(id) => Some(id),
                _ => None,
            });

            let mut physical_addr = relay_addr.clone();
            if let Some(Protocol::P2pCircuit) = physical_addr.pop() {
                println!("Dialing physical Relay address: {:?}", physical_addr);
                if let Err(e) = swarm.dial(physical_addr) {
                    println!("Error dialing Relay: {:?}", e);
                }
            } else {
                println!("Address did not end in p2p-circuit, dialing original...");
                if let Err(e) = swarm.dial(relay_addr.clone()) {
                    println!("Error dialing Relay: {:?}", e);
                }
            }
        }
    }

    // Connect to bootstrap nodes
    for addr_str in config.bootstrap_nodes {
        if let Ok(mut addr) = addr_str.parse::<libp2p::Multiaddr>() {
            if let Some(Protocol::P2p(remote_peer_id)) = addr.pop() {
                swarm.behaviour_mut().kademlia.add_address(&remote_peer_id, addr.clone());
                if Some(remote_peer_id) != relay_peer_id {
                    println!("Dialing bootstrap node: {:?}", addr);
                    let full_addr = addr.with(Protocol::P2p(remote_peer_id));
                    if let Err(e) = swarm.dial(full_addr) {
                         println!("Error dialing bootstrap: {:?}", e);
                    }
                } else {
                    println!("Bootstrap node is the Relay. Skipping double dial.");
                }
            }
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<(String, String)>();
    let _ = COMMAND_SENDER.set(tx.clone());
    let tx_inner = tx.clone();
    let mut peers_last_seen: HashMap<PeerId, Instant> = HashMap::new();
    let mut discovery_interval = tokio::time::interval(Duration::from_secs(15));

    // Event loop
    loop {
        tokio::select! {
            // Command from Flutter
            Some((recipient, msg_to_send)) = rx.recv() => {
                if recipient == "REFRESH" {
                    println!("Refreshing node discovery...");
                    peers_last_seen.clear();

                    // Refresh network
                    let _ = swarm.behaviour_mut().kademlia.bootstrap();

                    // Refresh mDNS
                    let random_peer = PeerId::random();
                    swarm.behaviour_mut().kademlia.get_closest_peers(random_peer);

                    // Announce presence via GossipSub
                    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
                    let msg = "ANNOUNCE:REFRESH";
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg.as_bytes()) {
                         println!("Error publishing refresh: {:?}", e);
                    }

                    // Send known peers to Flutter
                    for peer_id in swarm.connected_peers() {
                        if Some(*peer_id) != relay_peer_id {
                            if peers_last_seen.insert(*peer_id, Instant::now()).is_none() {
                                let _ = sink.add(format!("PEER+:{}", peer_id));
                            }
                        }
                    }
                } else {
                    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
                    // Publish message
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg_to_send.as_bytes()) {
                        println!("Publish error: {e:?}");
                    } else {
                        // Send ACK to Flutter
                        if recipient != "BROADCAST" {
                            let _ = sink.add(format!("MSG_SENT:{}:{}", recipient, msg_to_send));
                        }
                    }
                }
            }

            _ = discovery_interval.tick() => {
                if let Some(_) = relay_peer_id {
                    // Refresh Kademlia
                    let random_peer = PeerId::random();
                    swarm.behaviour_mut().kademlia.get_closest_peers(random_peer);

                    // Keep Gossipsub alive (heartbeat)
                    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
                    let msg = "ANNOUNCE:REFRESH";

                    let _ = swarm.behaviour_mut().gossipsub.publish(topic, msg.as_bytes());

                    // Clean Peers
                    let now = Instant::now();
                    let timeout = Duration::from_secs(45);

                    let dead_peers: Vec<PeerId> = peers_last_seen.iter()
                        .filter(|(_, &last_seen)| now.duration_since(last_seen) > timeout)
                        .map(|(&id, _)| id)
                        .collect();

                    for peer_id in dead_peers {
                        println!("Peer timed out (Zombie): {}", peer_id);
                        if peers_last_seen.remove(&peer_id).is_some() {
                            let _ = sink.add(format!("PEER-:{}", peer_id));
                        }
                    }
                }
            }

            // Network events
            event = swarm.select_next_some() => match event {
                // Capture Relay Client Events for debugging
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::RelayClient(event)) => {
                    match event {
                        relay::client::Event::ReservationReqAccepted { .. } => {
                            println!("RELAY: Reservation ACCEPTED! I am now reachable via the server.");

                            // Delay for Gossipsub initialization
                            let tx_for_task = tx_inner.clone();
                            tokio::spawn(async move {
                                tokio::time::sleep(Duration::from_millis(500)).await;
                                // Send message to ourselves (the event will send it properly)
                                if let Err(e) = tx_for_task.send(("BROADCAST".to_string(), "ANNOUNCE:PRESENCE".to_string())) {
                                    println!("Error sending delayed presence: {:?}", e);
                                }
                            });
                        },
                        other => {
                            println!("RELAY Event (Posible Error): {:?}", other);
                        },
                    }
                },

                // Receive message from Peer
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: _,
                    message,
                })) => {
                    let text = String::from_utf8_lossy(&message.data);
                    println!("Message received from {peer_id}: {text}");

                    if peers_last_seen.insert(peer_id, Instant::now()).is_none() {
                        println!("New peer discovered via Gossipsub: {}", peer_id);
                        let _ = sink.add(format!("PEER+:{}", peer_id));
                    }

                    // Handshake protocol logic
                    if text.starts_with("ANNOUNCE:PRESENCE") || text.starts_with("ANNOUNCE:REFRESH") {
                        println!("Presence signal from {}. Sending WELCOME back.", peer_id);

                        let topic = gossipsub::IdentTopic::new("p2p-chat-global");
                        let msg = "ANNOUNCE:WELCOME";
                        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg.as_bytes()) {
                            println!("Error sending welcome: {:?}", e);
                        }
                    } else if text.starts_with("ANNOUNCE:WELCOME") {
                         println!("Peer {} welcomed us. Connection established.", peer_id);
                    } else {
                        // Regular chat message
                        let _ = sink.add(format!("MSG:{}:{}", peer_id, text));
                    }
                },

                // Peer discovered (mDNS)
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        if peers_last_seen.insert(peer_id, Instant::now()).is_none() {
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                            println!("New peer discovered via mDNS: {}", peer_id);
                            let _ = sink.add(format!("PEER+:{}", peer_id));
                        }
                    }
                },

                // Peer discovered (Kademlia)
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Kademlia(libp2p::kad::Event::RoutingUpdated { peer, .. })) => {
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                    if Some(peer) != relay_peer_id {
                        if peers_last_seen.insert(peer, Instant::now()).is_none() {
                            println!("New peer discovered via Kademlia: {}", peer);
                            let _ = sink.add(format!("PEER+:{}", peer));
                        }
                    } else {
                        println!("Connection opened with Relay Server using Kademlia (Hidden from UI)");
                    }
                },

                // Any connection
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    if Some(peer_id) != relay_peer_id {
                        if peers_last_seen.insert(peer_id, Instant::now()).is_none() {
                            println!("New peer discovered: {}", peer_id);
                            let _ = sink.add(format!("PEER+:{}", peer_id));
                        }
                    } else {
                        println!("Connection established with Relay Server! Requesting Reservation...");
                        if let Some(addr) = &relay_address_to_dial {
                            if let Err(e) = swarm.listen_on(addr.clone()) {
                                println!("Error requesting reservation: {:?}", e);
                            }
                        }
                    }
                },

                // Peer disconnected
                SwarmEvent::ConnectionClosed { peer_id, num_established, .. } => {
                    if num_established == 0 {
                        println!("Connection closed with {peer_id}");
                        // Update Flutter
                        if peers_last_seen.remove(&peer_id).is_some() {
                            println!("Peer disconnected: {}", peer_id);
                            let _ = sink.add(format!("PEER-:{peer_id}"));
                        }

                        // Clear Gossipsub
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },

                // Peer expired
                SwarmEvent::Behaviour(MyP2PBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
                        if peers_last_seen.remove(&peer_id).is_some() {
                            println!("mDNS expired for {}: address {} removed form routing table", peer_id, multiaddr);
                            let _ = sink.add(format!("PEER-:{}", peer_id));
                        }
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
