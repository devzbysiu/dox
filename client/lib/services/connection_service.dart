import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/service_locator.dart';

typedef VoidFunction = void Function()?;

const filename = 'filename';
const thumbnail = 'thumbnail';
const fileUrl = 'fileUrl';
const thumbnailUrl = 'thumbnailUrl';

class ConnService {
  late final Stream _stream;

  ConnService({
    EventsStream? eventsStream,
  }) {
    final stream = eventsStream ?? getIt<EventsStream>();
    _stream = stream.stream; // TODO: improve this repetition
  }

  void onNewDoc(Function onNewDoc) {
    _stream.listen((data) {
      if (data == "new-doc") {
        onNewDoc();
      }
    });
  }

  void onConnected(Function onConnected) {
    _stream.listen((data) {
      if (data == "connected") {
        onConnected();
      }
    });
  }

  void onDone(VoidFunction onDone) {
    _stream.listen((_) {}, onDone: onDone);
  }
}
