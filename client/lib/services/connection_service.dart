import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/exceptions.dart';

const filename = 'filename';
const thumbnail = 'thumbnail';
const fileUrl = 'fileUrl';
const thumbnailUrl = 'thumbnailUrl';

typedef VoidFunction = void Function()?;

class ConnService {
  late final Stream _stream;

  static ConnService? _instance;

  static init(EventsStream stream) {
    _instance ??= ConnService._(stream);
  }

  ConnService._(EventsStream stream) {
    _stream = stream.stream; // TODO: improve this repetition
  }

  factory ConnService() {
    if (_instance == null) throw ApiNotInitializedException();
    return _instance!;
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