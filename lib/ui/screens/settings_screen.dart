import 'package:flutter/material.dart';
import '../../logic/node_manager.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  final TextEditingController _relayController = TextEditingController();
  final TextEditingController _bootstrapController = TextEditingController();
  final nodeManager = NodeManager();

  @override
  void initState() {
    super.initState();
    _relayController.text = nodeManager.customRelayAddress;
    _bootstrapController.text = nodeManager.customBootstrapNodes.join('\n');
  }

  @override
  void dispose() {
    _relayController.dispose();
    _bootstrapController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text("Network Settings")),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text("Relay Node Address", style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            TextField(
              controller: _relayController,
              decoration: const InputDecoration(
                hintText: "/ip4/X.X.X.X/tcp/4001/p2p/ID...",
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.router),
              ),
            ),
            const SizedBox(height: 24),
            const Text("Bootstrap Nodes", style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            TextField(
              controller: _bootstrapController,
              maxLines: 5,
              decoration: const InputDecoration(
                hintText: "One Multiaddr per line...",
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.group_work),
              ),
            ),
            const SizedBox(height: 30),
            Center(
              child: SizedBox(
                width: double.infinity,
                height: 50,
                child: ElevatedButton.icon(
                  onPressed: () async {
                    final relay = _relayController.text.trim();
                    final bootstraps = _bootstrapController.text
                        .split('\n')
                        .map((line) => line.trim())
                        .where((line) => line.isNotEmpty)
                        .toList();

                    await nodeManager.saveNewConfig(
                      relayBaseAddress: relay,
                      bootstrapNodes: bootstraps,
                    );

                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text("Saved! Restart app to apply.")),
                      );
                    }
                  },
                  icon: const Icon(Icons.save),
                  label: const Text("Save Configuration"),
                ),
              ),
            ),
            const SizedBox(height: 50),
            // Delete caht history
            const Divider(color: Colors.red),
            const Text(
              "Caution",
              style: TextStyle(
                color: Colors.red,
                fontWeight: FontWeight.bold,
                fontSize: 16
              ),
            ),
            const SizedBox(height: 10),

            SizedBox(
              width: double.infinity,
              height: 50,
              child: ElevatedButton.icon(
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.red[50],
                  foregroundColor: Colors.red,
                  elevation: 0,
                  side: const BorderSide(color: Colors.red),
                ),
                icon: const Icon(Icons.delete_forever),
                label: const Text("Clear All Chat History"),
                onPressed: () {
                  _showDeleteConfirmation(context);
                },
              ),
            ),
            const SizedBox(height: 30),
          ],
        ),
      ),
    );
  }

  void _showDeleteConfirmation(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text("Delete all adta?"),
        content: const Text(
          "This will permanently delete all message history from this device.\n\nThis action cannot be undone.",
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text("Cancel"),
          ),
          TextButton(
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            onPressed: () async {
              Navigator.of(ctx).pop();

              await nodeManager.clearAllData();

              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text("All history deleted.")),
                );
              }
            },
            child: const Text("Delete"),
          ),
        ],
      ),
    );
  }
}
