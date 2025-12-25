# Distributed P2P Messenger (Prototype)

A fully distributed, serverless messaging application built for privacy, resilience, and battery efficiency. This project creates a private overlay network where users communicate directly (Peer-to-Peer) without relying on central servers for message storage or routing.

> **Note:** This project is part of my engineering portfolio, exploring advanced concepts in decentralized systems, cryptography, and memory-safe programming.

## 🚀 Project Overview

The goal is to implement a robust decentralized communication tool that resists censorship and ensures data sovereignty. Unlike traditional client-server architectures, this application empowers users to own their identity and data.

Currently, the prototype demonstrates a functional **Local Mesh Network** capability, allowing devices on the same Wi-Fi to discover each other and chat instantly without internet access.

### Key Engineering Features

* **🛡️ Zero-Trust Security:**
    * **Transport Encryption:** All connections are secured using the **Noise Protocol Framework** (via `libp2p-noise`), ensuring data confidentiality on the wire.
    * *(Planned)* **Double Ratchet Algorithm:** For End-to-End Encryption (E2EE) and Perfect Forward Secrecy.
* **⚡ Hybrid Bootstrapping (Connectivity Layer):**
    * **Local (mDNS):** Implemented. Automatic discovery of peers on the same LAN/Wi-Fi (Zero-config offline messaging).
    * *(Planned)* **Global (DHT):** Kademlia DHT for global routing.
* **📡 Efficient Messaging:**
    * **GossipSub:** Uses a pub/sub architecture for efficient message propagation across the mesh network.
* **🏗️ Clean Architecture:**
    * Separation of concerns using a Singleton `NodeManager` for logic and reactive UI updates in Flutter.

## ✅ Implementation Status

This project is currently in the **Alpha Prototype** phase (Phase 1 Completed).

* [x] **Core Setup:** Rust + Flutter environment integration via `flutter_rust_bridge` (v2).
* [x] **Identity:** Secure `Ed25519` keypair generation (Randomized).
* [x] **Networking:** Instantiation of the `libp2p` swarm with TCP transport, Yamux multiplexing, and Noise encryption.
* [x] **Discovery (LAN):** Functional **mDNS** implementation. Nodes automatically detect entering/leaving peers.
* [x] **Messaging:** Full **GossipSub** integration. Text messages are broadcasted and acknowledged (`ACK`) by the Rust backend.
* [x] **Connection Lifecycle:** Instant detection of peer disconnection (TCP reset) with immediate UI feedback.
* [x] **UI/UX:** WhatsApp-style chat interface with conditional rendering and state management.
* [ ] **Persistence:** SQLite integration for chat history (Next Step).
* [ ] **Global Discovery:** Kademlia DHT implementation.

## 🛠️ The Tech Stack

Leveraging the "Golden Stack" for high-performance cross-platform engineering:

| Logical Layer | Technology / Crate | Technical Description |
| :--- | :--- | :--- |
| **Identity** | `libp2p::identity` | Ed25519 Keypair generation for node identification. |
| **Networking** | `libp2p` | Modular P2P networking stack (Swarm, Transport, Behaviour). |
| **Encryption** | `libp2p-noise` | **Transport Layer:** Noise Protocol handshake for encrypted connections. |
| **Discovery** | `libp2p-mdns` | Multicast DNS for finding peers on the local network. |
| **Messaging** | `libp2p-gossipsub` | Pub/Sub protocol for decentralized message routing. |
| **Concurrency** | `tokio` | Asynchronous runtime for handling non-blocking network I/O and channels. |
| **Bridge** | `flutter_rust_bridge` | Zero-copy FFI binding generation between Dart and Rust. |
| **UI Framework** | `Flutter` | Reactive UI with `AnimatedBuilder` state management. |

## 📂 Project Structure

The codebase follows a modular architecture separating the UI, Business Logic, and Systems Engineering:

```text
├── lib/                     # Flutter Code (Dart)
│   ├── logic/
│   │   └── node_manager.dart # Business Logic & State Management (Singleton)
│   ├── main.dart            # UI Components (Screens & Widgets)
│   └── src/                 # Auto-generated Rust bindings
├── rust/                    # Rust Core Logic (Crate)
│   ├── src/
│   │   ├── api/
│   │   │   └── node.rs      # P2P Node implementation (Swarm, Behaviour, Loop)
│   │   └── lib.rs           # Crate root
│   └── Cargo.toml           # Rust dependencies

```

## 🚦 Getting Started

### Prerequisites

* Flutter SDK
* Rust Toolchain (Cargo)
* *Windows/Linux/macOS for desktop debugging*

### Running the App

1. **Clone the repository:**

```bash
git clone https://github.com/Oiertxo/P2PMsg.git
cd P2PMsg

```

2. **Install dependencies:**

```bash
flutter pub get

```

3. **Run generation script (if needed):**

```bash
flutter_rust_bridge_codegen generate

```

4. **Launch (Run two instances to test):**

```bash
flutter run -d windows
# Open a new terminal
flutter run -d windows

```
