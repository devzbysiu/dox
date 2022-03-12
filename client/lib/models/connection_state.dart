import 'package:dox/utilities/api.dart';
import 'package:flutter/material.dart';

class AppState extends ChangeNotifier {
  late final Api _api;

  bool _isConnected = false;

  AppState(Api api) {
    _api = api;
    _api.onNewData(
      onDone: _notifyDisconnected,
      onConnected: _notifyConnected,
    );
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
