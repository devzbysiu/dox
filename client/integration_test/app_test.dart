import 'package:dox/main.dart' as app;
import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'test_utils.dart';

late final DoxMock _doxMock;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    _doxMock = await DoxMock.init();
  });

  tearDown(() {
    unregisterServices(); // so they can be registered again in next test
  });

  testWidgets('initially there are no documents displayed', (tester) async {
    // given
    _doxMock.serveEmptyDocumentsList();

    // when
    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsNothing);
  });

  testWidgets('all in-stage documents are displayed', (tester) async {
    // given
    _doxMock
      ..serveAllDocumentsList()
      ..servePlaceholderImages(8);

    // when
    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsWidgets);
  });
}
