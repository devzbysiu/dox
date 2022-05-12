import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/cupertino.dart';
import 'package:web_socket_channel/io.dart';

abstract class Connection implements ChangeNotifier {
  Stream get stream;

  void reconnect();

  void disconnect();
}

class ConnectionImpl extends ChangeNotifier with Log implements Connection {
  ConnectionImpl({
    Urls? urlsProvider,
  }) {
    final urls = urlsProvider ?? getIt<Urls>();
    _notificationsUri = urls.notifications();
    log.fine('initializing EventsStream with URL: "$_notificationsUri"');
    _connect();
  }

  void _connect() {
    _channel = IOWebSocketChannel.connect(_notificationsUri);
    _stream = _channel.stream.asBroadcastStream();
  }

  @override
  void reconnect() {
    _connect();
    notifyListeners();
  }

  @override
  void disconnect() {
    _channel.sink.close();
  }

  late IOWebSocketChannel _channel;

  late Stream _stream;

  late final Uri _notificationsUri;

  @override
  Stream get stream => _stream;
}
