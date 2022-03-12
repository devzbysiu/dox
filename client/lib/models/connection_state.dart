import 'package:dox/services/docs_service.dart';
import 'package:flutter/material.dart';

class ConnState extends ChangeNotifier {
  late final DocsService _api;

  bool _isConnected = false;

  ConnState(DocsService api) {
    _api = api;
    _api.onConnected(_notifyConnected);
    _api.onDone(_notifyDisconnected);
  }

  bool get isConnected => _isConnected;

  void _notifyDisconnected() {
    _isConnected = false;
    notifyListeners();
  }

  void _notifyConnected() {
    _isConnected = true;
    notifyListeners();
  }
}
