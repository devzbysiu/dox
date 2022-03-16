import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';

typedef OnConnected = void Function();
typedef OnDone = void Function()?;

class ConnService with Log {
  late final Stream _stream;

  ConnService({
    Events? ev,
  }) {
    log.fine('initializing ConnService');
    final events = ev ?? getIt<Events>();
    _stream = events.stream;
  }

  void onConnected(OnConnected onConnected) {
    log.fine('setting onConnected handler');
    _stream.listen((data) {
      if (data == "connected") {
        log.fine('connected event received, calling handler');
        onConnected();
      }
    });
  }

  void onDone(OnDone onDone) {
    log.fine('setting onDone handler');
    _stream.listen((_) {}, onDone: onDone);
  }
}