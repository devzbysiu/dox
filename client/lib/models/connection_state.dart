import 'package:dox/services/connection_service.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:flutter/material.dart';

class ConnState extends ChangeNotifier {
  late final ConnService _connService;

  bool _isConnected = false;

  ConnState({
    ConnService? connService,
  }) {
    _connService = connService ?? getIt<ConnService>();
    _connService.onConnected(_notifyConnected);
    _connService.onDone(_notifyDisconnected);
  }

  void _notifyDisconnected() {
    _isConnected = false;
    notifyListeners();
  }

  void _notifyConnected() {
    _isConnected = true;
    notifyListeners();
  }

  bool get isConnected => _isConnected;
}
