import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/cupertino.dart';
import 'package:web_socket_channel/io.dart';

abstract class Connection implements ChangeNotifier {
  void reconnect();

  void disconnect();

  void onConnected(Function fun);

  void onNewDoc(Function fun);

  void onDisconnected(Function fun);
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

  @override
  void onConnected(Function fun) {
    _stream.listen((data) {
      if (data == 'connected') {
        log.fine('connected, calling handler');
        fun();
      }
    });
  }

  @override
  void onNewDoc(Function fun) {
    _stream.listen((data) {
      if (data == 'new-doc') {
        log.fine('new document available, calling handler');
        fun();
      }
    });
  }

  @override
  void onDisconnected(Function fun) {
    _stream.listen((_) {}, onDone: () {
      log.fine('disconnected, calling handler');
      fun();
    });
  }

  late IOWebSocketChannel _channel;

  late Stream _stream;

  late final Uri _notificationsUri;
}
