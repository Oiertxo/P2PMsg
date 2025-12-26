import 'package:flutter/foundation.dart';
import 'package:p2p_msg/src/rust/api/node.dart';
import 'package:p2p_msg/logic/database_helper.dart';
import 'package:path_provider/path_provider.dart';

class NodeManager extends ChangeNotifier {
  // Singleton
  static final NodeManager _instance = NodeManager._internal();
  factory NodeManager() => _instance;
  NodeManager._internal();

  // Data state
  String myPeerId = "Initializing...";
  final List<String> peers = [];
  final Map<String, List<Map<String, String>>> chatHistory = {};
  final Set<String> unreadPeers = {};
  String? activeChatPeerId;

  bool _isNodeStarted = false;

  Future<void> start(String instanceName) async {
    if (_isNodeStarted) return;
    _isNodeStarted = true;
    print("Initializing instance (Runtime): $instanceName");

    // Get DB path
    final appDocDir = await getApplicationSupportDirectory();
    final storagePath = appDocDir.path;    
    print("App Storage Path: $storagePath");

    // Init DB
    await DatabaseHelper().init(storagePath, instanceName);
    // Load history from DB
    await _loadHistory();

    // Start node
    final stream = startP2PNode(storagePath: storagePath, instanceName: instanceName);
    stream.listen((message) {
      _processMessage(message);
    });
  }

  // Recovers messages from DB
  Future<void> _loadHistory() async {
    final history = await DatabaseHelper().getAllMessages();
    
    for (var row in history) {
      final peerId = row['peerId'] as String;
      final sender = row['sender'] as String;
      final text = row['text'] as String;
      
      if (!chatHistory.containsKey(peerId)) {
        chatHistory[peerId] = [];
      }
      chatHistory[peerId]!.add({'sender': sender, 'text': text});
    }
    // Notify UI
    notifyListeners();
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
        // Update state
        _addMessageToHistory(sender, {'sender': 'peer', 'text': text});
        if (sender != activeChatPeerId) {
          unreadPeers.add(sender);
        }
        // Store message in DB
        DatabaseHelper().insertMessage(sender, 'peer', text);
        needsUpdate = true;
      }
    }
    // Message sent
    else if (message.startsWith("MSG_SENT:")) {
      final parts = message.split(":");
      if (parts.length >= 3) {
        final recipientId = parts[1];
        final text = parts.sublist(2).join(":");
        // Update state
        _addMessageToHistory(recipientId, {'sender': 'me', 'text': text});
        // Store message in DB
        DatabaseHelper().insertMessage(recipientId, 'me', text);
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