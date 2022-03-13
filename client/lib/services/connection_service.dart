import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';

typedef VoidFunction = void Function()?;

const filename = 'filename';
const thumbnail = 'thumbnail';
const fileUrl = 'fileUrl';
const thumbnailUrl = 'thumbnailUrl';

class ConnService with Log {
  late final Stream _stream;

  ConnService({
    EventsStream? eventsStream,
  }) {
    log.fine('initializing ConnService');
    final stream = eventsStream ?? getIt<EventsStream>();
    _stream = stream.stream; // TODO: improve this repetition
  }

  void onConnected(Function onConnected) {
    log.fine('setting onConnected handler');
    _stream.listen((data) {
      if (data == "connected") {
        log.fine('connected event received, calling handler');
        onConnected();
      }
    });
  }

  void onDone(VoidFunction onDone) {
    log.fine('setting onDone handler');
    _stream.listen((_) {}, onDone: onDone);
  }
}
