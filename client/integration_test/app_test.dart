import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:dox/main.dart' as app;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('initially there are no documents displayed', (tester) async {
    // given
    app.main();
    await tester.pumpAndSettle();

    // then
    expect(find.byType(OpenableDocument), findsNothing);
  });
}
