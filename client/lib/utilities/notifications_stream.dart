import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/cupertino.dart';
import 'package:web_socket_channel/io.dart';

abstract class NotificationsStream implements ChangeNotifier {
  Stream get stream;
  void reconnect();
  void disconnect();
}

class NotificationsStreamImpl extends ChangeNotifier
    with Log
    implements NotificationsStream {
  NotificationsStreamImpl({
    Urls? urlsProvider,
  }) {
    final urls = urlsProvider ?? getIt<Urls>();
    _notificationsUri = urls.notifications();
    log.fine('initializing EventsStream with URL: "$_notificationsUri"');
    _channel = _connect();
    _stream = _channel.stream.asBroadcastStream();
  }

  IOWebSocketChannel _connect() {
    return IOWebSocketChannel.connect(_notificationsUri);
  }

  @override
  void reconnect() {
    _channel = _connect();
    _stream = _channel.stream.asBroadcastStream();
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
