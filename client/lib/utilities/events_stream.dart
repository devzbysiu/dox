import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:web_socket_channel/io.dart';

abstract class Events {
  Stream get stream;
}

class EventsImpl with Log implements Events {
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

  late final Stream _stream;

  late final Uri _notificationsUri;

  @override
  Stream get stream => _stream;
}
