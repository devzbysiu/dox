import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:web_socket_channel/io.dart';

class EventsStream {
  late final Stream _stream;

  EventsStream({
    Urls? urlsProvider,
  }) {
    final urls = urlsProvider ?? getIt<Urls>();
    _stream = IOWebSocketChannel.connect(urls.notifications())
        .stream
        .asBroadcastStream();
  }

  Stream get stream => _stream;
}
