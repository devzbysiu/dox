import 'dart:io';
import 'dart:typed_data';

import 'package:dox/main.dart' as app;
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:mock_web_server/mock_web_server.dart';

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
  @override
  Stream get stream => const Stream.empty();
}

late final MockWebServer _server;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUp(() async {
    _server = MockWebServer();
    await _server.start();
    app.configOverride = MockConfig(_server.url, _server.url);
    app.eventsOverride = EventsMock();
  });

  testWidgets('initially there are no documents displayed', (tester) async {
    // given
    _server.enqueue(body: _emptyDocumentsList());
    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsNothing);
  });

  testWidgets('all in-stage documents are displayed', (tester) async {
    // given
    _server.enqueue(
      headers: {"Content-Type": "application/json"},
      body: _allDocumentsList(),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/png"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc1.png"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/jpeg"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc2.jpg"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/jpeg"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc3.jpg"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/webp"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc4.webp"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/jpeg"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc5.jpg"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/jpeg"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc6.jpg"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/jpeg"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc7.jpg"),
    );

    _server.enqueue(
      headers: {"Content-Type": "image/jpeg"},
      body: _readFileByte("/home/zbyniu/Projects/dox/core/res/doc8.jpg"),
    );

    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsNWidgets(6));
  });
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

Uint8List _readFileByte(String filePath) {
  return Uint8List.fromList(List.of([1, 2, 3]));
}