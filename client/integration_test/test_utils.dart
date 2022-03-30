import 'dart:async';

import 'package:dox/main.dart' as app;
import 'package:dox/services/connection_service.dart';
import 'package:dox/services/doc_scan_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:http/http.dart' as http;
import 'package:mock_web_server/mock_web_server.dart';
import 'package:stream_channel/stream_channel.dart';

class MockConfig implements Config {
  MockConfig(this.base, this.websocket);

  final String base;

  final String websocket;

  @override
  String get baseUrl => base;

  @override
  String get websocketUrl => websocket;
}

class EventsMock implements Events {
  EventsMock() {
    _streamController = StreamController();
    final sinkController = StreamController();
    final channel = StreamChannel(
      _streamController.stream,
      sinkController.sink,
    );
    _stream = channel.stream.asBroadcastStream();
  }

  late final StreamController _streamController;

  late final Stream _stream;

  void cause(Event event) {
    _streamController.add(event.string());
  }

  @override
  Stream get stream => _stream;
}

enum Event { connected }

extension EventExt on Event {
  String string() {
    return name.toString().split('.').last;
  }
}

class DoxMock {
  static Future<DoxMock> init() async {
    final server = MockWebServer();
    await server.start();
    return DoxMock._(server, EventsMock());
  }

  DoxMock._(MockWebServer server, EventsMock eventsMock) {
    _server = server;
    app.configOverride = MockConfig(server.url, server.url);

    _events = eventsMock;
    app.eventsOverride = eventsMock;
  }

  late final MockWebServer _server;

  late final EventsMock _events;

  void serveEmptyDocumentsList() {
    _server.enqueue(body: _emptyDocumentsList());
  }

  void serveAllDocumentsList() {
    _server.enqueue(
      headers: {"Content-Type": "application/json"},
      body: _allDocumentsList(),
    );
  }

  void servePlaceholderImages(int n) async {
    var body = await _placeholderImage();
    for (var i = 0; i < n; i++) {
      _server.enqueue(
        headers: {"Content-Type": "image/png"},
        body: body,
      );
    }
  }

  EventsMock get events => _events;
}

void unregisterServices() {
  getIt.unregister<Config>();
  getIt.unregister<Urls>();
  getIt.unregister<Events>();
  getIt.unregister<DocsService>();
  getIt.unregister<ConnService>();
  getIt.unregister<DocScanService>();
}

String _emptyDocumentsList() => '{ "entries": []}';

String _allDocumentsList() => '''{
  "entries": [
    {
      "filename": "doc8.jpg",
      "thumbnail": "doc8.jpg"
    },
    {
      "filename": "doc5.jpg",
      "thumbnail": "doc5.jpg"
    },
    {
      "filename": "doc3.jpg",
      "thumbnail": "doc3.jpg"
    },
    {
      "filename": "doc1.png",
      "thumbnail": "doc1.png"
    },
    {
      "filename": "doc7.jpg",
      "thumbnail": "doc7.jpg"
    },
    {
      "filename": "doc2.jpg",
      "thumbnail": "doc2.jpg"
    },
    {
      "filename": "doc6.jpg",
      "thumbnail": "doc6.jpg"
    },
    {
      "filename": "doc4.webp",
      "thumbnail": "doc4.webp"
    }
  ]
}
''';

Future<String> _placeholderImage() async {
  final res = await http.get(Uri.parse('https://via.placeholder.com/320'));
  return res.body;
}
