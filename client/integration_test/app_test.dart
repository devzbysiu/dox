import 'package:dox/main.dart' as app;
import 'package:dox/widgets/document/openable_document.dart';
import 'package:dox/widgets/status_dot.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:mock_web_server/mock_web_server.dart';

import '../test/utils.dart';
import 'test_utils.dart';

late final MockWebServer _server;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    _server = await mockDoxService();
  });

  tearDown(() {
    unregisterServices(); // so they can be registered again in next test
  });

  testWidgets('initially there are no documents displayed', (tester) async {
    // given
    _server.serveEmptyDocumentsList();

    // when
    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsNothing);
  });

  testWidgets('all in-stage documents are displayed', (tester) async {
    // given
    _server
      ..serveAllDocumentsList()
      ..servePlaceholderImages(8);

    // when
    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsWidgets);
  });

  testWidgets('the StatusDot is gray when no connection', (tester) async {
    // given
    _server.serveEmptyDocumentsList(); // not important

    // when
    app.main();
    await tester.pumpAndSettle();
    final StatusDot statusDot = tester.firstWidget(find.byType(StatusDot));

    // then
    expect(statusDot.color(tester), equals([Colors.blueGrey, Colors.blueGrey]));
  });

  testWidgets('StatusDot changes to green when connected', (tester) async {
    // given
    _server.serveEmptyDocumentsList(); // not important
    final eventsMock = EventsMock();
    app.eventsOverride = eventsMock;
    app.main();
    await tester.pumpAndSettle();
    final StatusDot statusDot = tester.firstWidget(find.byType(StatusDot));
    expect(statusDot.color(tester), equals([Colors.blueGrey, Colors.blueGrey]));

    // when
    eventsMock.sendEvent(Event.connected);
    await tester.pumpAndSettle();

    // then
    expect(
      statusDot.color(tester),
      equals([Colors.green[300]!, Colors.yellow[400]!]),
    );
  });
}
