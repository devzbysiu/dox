import 'package:dox/services/connection_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:flutter/material.dart';

class ConnState extends ChangeNotifier with Log {
  late final ConnService _connService;

  bool _isConnected = false;

  ConnState({
    ConnService? connService,
  }) {
    log.fine('initializing ConnState');
    _connService = connService ?? getIt<ConnService>();
    _connService.onConnected(_notifyConnected);
    _connService.onDone(_notifyDisconnected);
  }

  void _notifyDisconnected() {
    log.fine('core disconnected, notifying');
    _isConnected = false;
    notifyListeners();
  }

  void _notifyConnected() {
    log.fine('core connected, notifying');
    _isConnected = true;
    notifyListeners();
  }

  bool get isConnected => _isConnected;
}
