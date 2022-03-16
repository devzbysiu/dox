import 'package:dox/models/connection_state.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/services/connection_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/urls.dart';
import 'package:dox/widgets/status_dot.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:get_it/get_it.dart';
import 'package:provider/provider.dart';

final getIt = GetIt.instance;

void main() {
  testWidgets('StatusDot initially displays gray dot', (tester) async {
    dotenv.testLoad(fileInput: '''
    BASE_URL=http://192.168.16.247:8000
    WEBSOCKET_URL=ws://192.168.16.247:8001
    ''');
    getIt.registerSingleton(await Config.init());
    getIt.registerSingleton(Urls());
    getIt.registerSingleton(Events());
    getIt.registerSingleton(ConnService());

    await tester.pumpWidget(MultiProvider(
      providers: [
        ChangeNotifierProvider<DocsState>(create: (_) => DocsState()),
        ChangeNotifierProvider<ConnState>(create: (_) => ConnState()),
      ],
      child: const StatusDot(),
    ));
    // final LinearGradient dot = tester.firstWidget(find.byType(LinearGradient));
    // expect(dot.colors, equals([Colors.blueGrey, Colors.blueGrey]));
  });
}
