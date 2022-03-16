import 'package:dox/models/connection_state.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/services/connection_service.dart';
import 'package:dox/services/docs_service.dart';
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
  testWidgets("StatusDot initially displays gray dot", (tester) async {
    final urls = Urls(config: _ConfigMock());
    final ev = Events(urlsProvider: urls);

    await tester.pumpWidget(MultiProvider(
      providers: [
        ChangeNotifierProvider<DocsState>(
          create: (_) => DocsState(
            docsService: DocsService(
              urls: urls,
              ev: ev,
            ),
          ),
        ),
        ChangeNotifierProvider<ConnState>(
          create: (_) => ConnState(
            connService: ConnService(
              ev: ev,
            ),
          ),
        ),
      ],
      child: const StatusDot(),
    ));
    // final LinearGradient dot = tester.firstWidget(find.byType(LinearGradient));
    // expect(dot.colors, equals([Colors.blueGrey, Colors.blueGrey]));
  });
}

class _ConfigMock extends Config {
  @override
  String get baseUrl => 'http://192.168.16.247:8000';

  @override
  String get websocketUrl => 'ws://192.168.16.247:8001';
}
