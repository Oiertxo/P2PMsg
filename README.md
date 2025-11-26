# Distributed P2P Messenger

A fully distributed, serverless messaging application built for privacy and resilience. This project creates a private overlay network where users communicate directly (Peer-to-Peer) without relying on central servers for message storage or routing.

## 🚀 Project Overview

The goal of this project is to implement a robust decentralized communication tool that resists censorship and ensures data sovereignty. Unlike traditional client-server architectures (e.g., WhatsApp, Telegram), this application treats every device as a node in a global network.

Key engineering features:

* **Serverless Architecture:** No central database. Messages live only on the users' devices.
* **Distributed Discovery:** Uses a Kademlia DHT (Distributed Hash Table) to resolve user identities to IP addresses without a central phonebook.
* **NAT Traversal:** Implements "Hole Punching" (DCUtR protocol) to allow direct connections between devices behind residential routers and firewalls.
* **End-to-End Encryption:** All traffic is secured using the Noise Protocol framework, ensuring that not even the relay nodes can read the content.

## 🛠️ The Tech Stack

This project leverages the "Golden Stack" for high-performance cross-platform development:

* **Core Logic & Networking:** [Rust](https://www.rust-lang.org/)
    * *Why?* Memory safety, zero-cost abstractions, and low-level control over system resources.
    * *Key Library:* `libp2p` (The modular network stack used by IPFS and Ethereum 2.0).
* **User Interface:** [Flutter](https://flutter.dev/) (Dart)
    * *Why?* Native performance on Windows, Android, and iOS with a single codebase.
* **Interoperability:** [flutter_rust_bridge (v2)](https://github.com/fzyzcjy/flutter_rust_bridge)
    * *Why?* Seamlessly connects the Dart UI with the Rust backend, handling complex type conversions and async execution automatically.