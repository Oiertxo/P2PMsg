import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../src/rust/node.dart' as rust;
import '../src/rust/config.dart';
import '../src/rust/frb_generated.dart';
import 'database_helper.dart';

class Message {
  final String peerId;
  final String content;
  final bool isMe;
  final int timestamp;

  Message(this.peerId, this.content, this.isMe, this.timestamp);
}

class NodeManager extends ChangeNotifier {
  // Singleton
  static final NodeManager _instance = NodeManager._internal();
  factory NodeManager() => _instance;
  NodeManager._internal();

  // State
  bool _isNodeStarted = false;
  final StreamController<String> _sink = StreamController.broadcast();

  // Data
  List<String> _peers = [];
  List<String> get peers => _peers;
  Map<String, List<Message>> _messages = {};
  Map<String, int> _unreadCounts = {};
  String? activeChatPeerId;
  final Set<String> _onlinePeers = {};
  bool isPeerOnline(String peerId) => _onlinePeers.contains(peerId);

  // Custom config
  String _customRelayAddress = "";
  List<String> _customBootstrapNodes = [];
  String get customRelayAddress => _customRelayAddress;
  List<String> get customBootstrapNodes => _customBootstrapNodes;

  // Start method
  Future<void> start(String instanceName) async {
    if (_isNodeStarted) return;
    _isNodeStarted = true;
    print("Initializing instance (Runtime): $instanceName");

    // Load config
    final AppConfig config = await _loadConfig();

    // Load history
    final appDocDir = await getApplicationSupportDirectory();
    final storagePath = appDocDir.path;
    await DatabaseHelper().init(storagePath, instanceName);
    await _loadKnownPeers();

    // Start Node
    final stream = rust.startP2PNode(
      storagePath: storagePath,
      instanceName: instanceName,
      config: config,
    );

    // Listen on stream
    stream.listen((String message) {
      _processMessage(message);
    });
  }

  // Load known peers for history
  Future<void> _loadKnownPeers() async {
    final storedPeers = await DatabaseHelper().getKnownPeers();

    for (var peerId in storedPeers) {
      if (!_peers.contains(peerId)) {
        _peers.add(peerId);
        await _loadChatForPeer(peerId);
      }
    }
    notifyListeners();
  }

  Future<void> _loadHistoryFromDB() async {
    final rows = await DatabaseHelper().getAllMessages();
    _messages = {};

    for (var row in rows) {
      final peerId = row['peerId'] as String;
      final sender = row['sender'] as String;
      final text = row['text'] as String;
      final timestamp = row['timestamp'] as int;
      final isMe = (sender == 'ME');

      final msg = Message(peerId, text, isMe, timestamp);

      if (!_messages.containsKey(peerId)) {
        _messages[peerId] = [];
      }
      _messages[peerId]!.add(msg);

      if (!_peers.contains(peerId)) {
        _peers.add(peerId);
      }
    }
    notifyListeners();
  }

  // Load only history of specific Peer
  Future<void> _loadChatForPeer(String peerId) async {
    if (_messages.containsKey(peerId) && _messages[peerId]!.isNotEmpty) return;
    final rows = await DatabaseHelper().getMessagesForPeer(peerId);
    if (rows.isEmpty) return;

    final List<Message> chatHistory = [];
    for (var row in rows) {
      final sender = row['sender'] as String;
      final text = row['text'] as String;
      final timestamp = row['timestamp'] as int;
      final isMe = (sender == 'ME');

      chatHistory.add(Message(peerId, text, isMe, timestamp));
    }

    _messages[peerId] = chatHistory;
    notifyListeners();
  }

  // Refresh node list
  void refreshNode() {
    print("Refreshing node network...");
    rust.refreshNode();
  }

  void sendMsg(String peerId, String msg) {
    rust.sendMessage(recipient: peerId, msg: msg);
    _storeMessage(peerId, msg, isMe: true);
  }

  List<dynamic> getMessages(String peerId) {
    return _messages[peerId] ?? [];
  }

  int unreadCount(String peerId) {
    return _unreadCounts[peerId] ?? 0;
  }

  void markAsRead(String peerId) {
    if (_unreadCounts.containsKey(peerId)) {
      _unreadCounts[peerId] = 0;
      notifyListeners();
    }
  }

  void _processMessage(String rawMsg) {
    // Expected format:
    // PEER+:12D3...
    // PEER-:12D3...
    // MSG:12D3...:Text
    // MSG_SENT:12D3...:Text (ACK)

    if (rawMsg.startsWith("PEER+:")) {
      final peerId = rawMsg.substring(6);
      _onlinePeers.add(peerId);
      if (!_peers.contains(peerId)) {
        _peers.add(peerId);
        _loadChatForPeer(peerId);
      }
      notifyListeners();
    }
    else if (rawMsg.startsWith("PEER-:")) {
      final peerId = rawMsg.substring(6);
      _onlinePeers.remove(peerId);
      notifyListeners();
    }
    else if (rawMsg.startsWith("MSG:")) {
      // MSG:PEER_ID:TEXT
      final parts = rawMsg.split(":");
      if (parts.length >= 3) {
        final peerId = parts[1];
        final content = parts.sublist(2).join(":");

        _storeMessage(peerId, content, isMe: false);

        if (activeChatPeerId != peerId) {
          _unreadCounts[peerId] = (_unreadCounts[peerId] ?? 0) + 1;
        }
        notifyListeners();
      }
    }
  }

  void _storeMessage(String peerId, String content, {required bool isMe}) {
    if (!_messages.containsKey(peerId)) {
      _messages[peerId] = [];
    }

    final newMsg = Message(
      peerId,
      content,
      isMe,
      DateTime.now().millisecondsSinceEpoch
    );

    _messages[peerId]!.add(newMsg);

    // Save to DB
    final senderStr = isMe ? "ME" : peerId;
    DatabaseHelper().insertMessage(peerId, senderStr, content);

    notifyListeners();
  }

  // Config
  Future<AppConfig> _loadConfig() async {
    final prefs = await SharedPreferences.getInstance();

    _customRelayAddress = prefs.getString('custom_relay_base') ?? "";
    _customBootstrapNodes = prefs.getStringList('custom_bootstraps') ?? [];

    if (_customRelayAddress.isNotEmpty || _customBootstrapNodes.isNotEmpty) {
      List<String> finalBootstraps = _customBootstrapNodes;
      if (finalBootstraps.isEmpty && _customRelayAddress.isNotEmpty) {
        finalBootstraps = [_customRelayAddress];
      }

      return AppConfig(
        isBootstrapNode: false,
        relayAddress: _customRelayAddress.isNotEmpty
            ? "$_customRelayAddress/p2p-circuit"
            : "",
        bootstrapNodes: finalBootstraps,
        listenPort: 0,
      );
    }

    try {
      final String response = await rootBundle.loadString('assets/config.json');
      final data = json.decode(response);
      return AppConfig(
        isBootstrapNode: data['is_bootstrap_node'] ?? false,
        relayAddress: data['relay_address'] ?? "",
        bootstrapNodes: List<String>.from(data['bootstrap_nodes'] ?? []),
        listenPort: data['listen_port'] ?? 0,
      );
    } catch (e) {
      print("Error loading config asset: $e");
      return AppConfig(
        isBootstrapNode: false,
        relayAddress: "",
        bootstrapNodes: [],
        listenPort: 0,
      );
    }
  }

  Future<void> saveNewConfig({
    required String relayBaseAddress,
    required List<String> bootstrapNodes
  }) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('custom_relay_base', relayBaseAddress);
    await prefs.setStringList('custom_bootstraps', bootstrapNodes);
    _customRelayAddress = relayBaseAddress;
    _customBootstrapNodes = bootstrapNodes;
    notifyListeners();
  }

  Future<void> clearAllData() async {
    await DatabaseHelper().deleteAllMessages();
    _messages.clear();
    _unreadCounts.clear();
    notifyListeners();
    print("All data cleared.");
  }
}
