import 'package:dox/utilities/urls.dart';
import 'package:get_it/get_it.dart';
import 'package:web_socket_channel/io.dart';

final getIt = GetIt.instance;

class EventsStream {
  late final Stream _stream;

  EventsStream({
    Urls? urlsProvider,
  }) {
    final urls = urlsProvider ?? getIt.get<Urls>();
    _stream = IOWebSocketChannel.connect(urls.notifications())
        .stream
        .asBroadcastStream();
  }

  Stream get stream => _stream;
}
