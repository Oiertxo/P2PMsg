import 'package:flutter/material.dart';
import 'package:p2pmsg/src/rust/api/simple.dart';
import 'package:p2pmsg/src/rust/frb_generated.dart';

Future<void> main() async {
  // Initialize Rust
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('P2P Messenger')),
        body: Center(
          child: IdentityWidget(),
        ),
      ),
    );
  }
}

class IdentityWidget extends StatefulWidget {
  const IdentityWidget({super.key});

  @override
  State<IdentityWidget> createState() => _IdentityWidgetState();
}

class _IdentityWidgetState extends State<IdentityWidget> {
  String _identity = "Generating keys...";

  @override
  void initState() {
    super.initState();
    _loadIdentity();
  }

  void _loadIdentity() {
    final id = generateMyIdentity();
    
    setState(() {
      _identity = id;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.security, size: 64, color: Colors.blue),
          const SizedBox(height: 20),
          const Text(
            "Data:",
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 10),
          SelectableText(
            _identity,
            textAlign: TextAlign.center,
            style: const TextStyle(fontSize: 16, fontFamily: 'Courier'),
          ),
        ],
      ),
    );
  }
}