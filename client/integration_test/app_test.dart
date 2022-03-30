import 'package:dox/main.dart' as app;
import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:mock_web_server/mock_web_server.dart';

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
}
