import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/cupertino.dart';
import 'package:web_socket_channel/io.dart';

abstract class Events implements ChangeNotifier {
  Stream get stream;
}

class EventsImpl extends ChangeNotifier with Log implements Events {
  EventsImpl({
    Urls? urlsProvider,
  }) {
    final urls = urlsProvider ?? getIt<Urls>();
    _notificationsUri = urls.notifications();
    log.fine('initializing EventsStream with URL: "$_notificationsUri"');
    _stream = _connect();
  }

  Stream _connect() {
    return IOWebSocketChannel.connect(_notificationsUri)
        .stream
        .asBroadcastStream();
  }

  void reconnect() {
    _stream = _connect();
    notifyListeners();
  }

  late final Stream _stream;

  late final Uri _notificationsUri;

  @override
  Stream get stream => _stream;
}
