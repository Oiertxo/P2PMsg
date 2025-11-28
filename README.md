# Distributed P2P Messenger (Prototype)

A fully distributed, serverless messaging application built for privacy and resilience. This project creates a private overlay network where users communicate directly (Peer-to-Peer) without relying on central servers for message storage or routing.

> **Note:** This project is part of my engineering portfolio, exploring advanced concepts in decentralized systems and memory-safe programming.

## 🚀 Project Overview

The goal is to implement a robust decentralized communication tool that resists censorship and ensures data sovereignty. Unlike traditional client-server architectures (e.g., WhatsApp, Telegram), this application treats every mobile device as a full node in a global network.

### Key Engineering Features

* **Serverless Architecture:** No central database. Messages live only on the users' devices.
* **Distributed Discovery:** Uses a **Kademlia DHT** (Distributed Hash Table) to resolve user identities to IP addresses without a central phonebook.
* **NAT Traversal:** Implements "Hole Punching" (DCUtR protocol) to allow direct connections between devices behind residential routers and firewalls.
* **End-to-End Encryption:** All traffic is secured using the **Noise Protocol** framework, ensuring that not even relay nodes can read the content.

## ✅ Current Implementation Status

This project is currently in the **Alpha Prototype** phase.

* [x] **Core Setup:** Rust + Flutter environment integration via `flutter_rust_bridge`.
* [x] **Identity Generation:** Secure creation of **Ed25519** keypairs for node identity.
* [x] **Swarm Initialization:** Successful instantiation of the `libp2p` swarm.
* [x] **Basic Connectivity:** Node can listen on local interfaces and identify its own Peer ID.
* [ ] **Discovery:** Kademlia DHT integration for peer finding (In Progress).
* [ ] **Messaging:** GossipSub implementation for chat rooms (Planned).

## 🛠️ The Tech Stack

Leveraging the "Golden Stack" for high-performance cross-platform engineering:

* **Core Logic & Networking:** [Rust](https://www.rust-lang.org/)
  * *Why?* Memory safety, zero-cost abstractions, and low-level control over system resources.
  * *Key Library:* `libp2p` (The modular network stack used by IPFS and Ethereum 2.0).
* **User Interface:** [Flutter](https://flutter.dev/) (Dart)
  * *Why?* Native performance on Windows, Android, and iOS with a single codebase.
* **Interoperability:** [flutter\_rust\_bridge](https://github.com/fzyzcjy/flutter_rust_bridge)
  * *Why?* Seamlessly connects the Dart UI with the Rust backend, handling complex type conversions and async execution automatically without manual FFI boilerplate.

## 📂 Project Structure

The codebase follows a modular architecture separating the UI from the heavy lifting:

```
├── lib/                 # Flutter UI Code (Dart)
│   ├── main.dart        # Entry point & UI Logic
│   └── bridge_generated # Auto-generated bindings
├── native/              # Rust Core Logic (Crate)
│   ├── src/
│   │   ├── api.rs       # Exposed API for Flutter
│   │   └── main.rs      # P2P Node implementation (libp2p)
│   └── Cargo.toml       # Rust dependencies
```

## 🚦 Getting Started

### Prerequisites

* Flutter SDK
* Rust Toolchain (Cargo)

### Running the App

1. **Clone the repository:**
    ```bash
    git clone https://github.com/oiera/p2p-messenger.git
    cd p2p-messenger
    ```
2. **Install dependencies:**
    ```bash
    flutter pub get
    ```
3. **Run generation script (for Rust bridge):**
    ```bash
    flutter_rust_bridge_codegen generate
    ```
4. **Launch:**
    ```bash
    flutter run
    ```
    *Check the console output to see your generated PeerID\!*
