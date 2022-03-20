import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:web_socket_channel/io.dart';

class Events with Log {
  Events({
    Urls? urlsProvider,
  }) {
    final urls = urlsProvider ?? getIt<Urls>();
    log.fine('initializing EventsStream with URL: "${urls.notifications()}"');
    _stream = IOWebSocketChannel.connect(urls.notifications())
        .stream
        .asBroadcastStream();
  }

  late final Stream _stream;

  Stream get stream => _stream;
}
