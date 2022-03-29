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
}

String _emptyDocumentsList() => '{ "entries": []}';
