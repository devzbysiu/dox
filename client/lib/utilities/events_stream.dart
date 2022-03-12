import 'package:web_socket_channel/io.dart';

class EventsStream {
  late final Stream _stream;

  EventsStream(Uri source) {
    _stream = IOWebSocketChannel.connect(source)
        .stream
        .asBroadcastStream();
  }

  Stream get stream => _stream;
}