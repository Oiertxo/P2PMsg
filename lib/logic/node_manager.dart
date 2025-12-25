import 'package:flutter/foundation.dart';
import 'package:p2pmsg/src/rust/api/node.dart';

class NodeManager extends ChangeNotifier {
  // Singleton
  static final NodeManager _instance = NodeManager._internal();
  factory NodeManager() => _instance;
  NodeManager._internal();

  // Data state
  String myPeerId = "Iniciando...";
  final List<String> peers = [];
  final Map<String, List<Map<String, String>>> chatHistory = {};
  final Set<String> unreadPeers = {};

  bool _isNodeStarted = false;

  void start() {
    if (_isNodeStarted) return;
    _isNodeStarted = true;

    // Rust call
    final stream = startP2PNode(); 

    stream.listen((message) {
      _processMessage(message);
    });
  }

  void sendMessageTo(String recipient, String text) {
    if (text.trim().isEmpty) return;
    
    sendMessage(recipient: recipient, msg: text);
  }

  void markAsRead(String peerId) {
    if (unreadPeers.contains(peerId)) {
      unreadPeers.remove(peerId);
      notifyListeners();
    }
  }

  void _processMessage(String message) {
    bool needsUpdate = false;

    // Self Peer ID
    if (message.startsWith("ME:")) {
      myPeerId = message.substring(3);
      needsUpdate = true;
    } 
    // New Peer connected
    else if (message.startsWith("PEER+:")) {
      final newPeer = message.substring(6);
      if (!peers.contains(newPeer)) {
        peers.add(newPeer);
        needsUpdate = true;
      }
    } 
    // Peer disconnected
    else if (message.startsWith("PEER-:")) {
      final lostPeer = message.substring(6);
      peers.remove(lostPeer);
      unreadPeers.remove(lostPeer);
      needsUpdate = true;
    } 
    // Message received from Peer
    else if (message.startsWith("MSG:")) {
      final parts = message.split(":");
      if (parts.length >= 3) {
        final sender = parts[1];
        final text = parts.sublist(2).join(":");

        _addMessageToHistory(sender, {'sender': 'peer', 'text': text});
        unreadPeers.add(sender);
        needsUpdate = true;
      }
    }
    // Message sent
    else if (message.startsWith("MSG_SENT:")) {
      final parts = message.split(":");
      if (parts.length >= 3) {
        final recipientId = parts[1];
        final text = parts.sublist(2).join(":");
        
        _addMessageToHistory(recipientId, {'sender': 'me', 'text': text});
        needsUpdate = true;
      }
    }

    if (needsUpdate) {
      notifyListeners();
    }
  }

  void _addMessageToHistory(String peerId, Map<String, String> msg) {
    if (!chatHistory.containsKey(peerId)) {
      chatHistory[peerId] = [];
    }
    chatHistory[peerId]!.add(msg);
  }
}