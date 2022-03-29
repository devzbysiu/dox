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
    log.fine('initializing EventsStream with URL: "${urls.notifications()}"');
    _stream = IOWebSocketChannel.connect(urls.notifications())
        .stream
        .asBroadcastStream();
  }

  late final Stream _stream;

  @override
  Stream get stream => _stream;
}
