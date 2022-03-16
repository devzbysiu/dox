import 'package:dox/models/connection_state.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/services/connection_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/urls.dart';
import 'package:dox/widgets/status_dot.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:get_it/get_it.dart';
import 'package:provider/provider.dart';

final getIt = GetIt.instance;

void main() {
  testWidgets("StatusDot initially displays gray dot", (tester) async {
    await tester.pumpWidget(_wrapper(child: const StatusDot()));
    final Container container = tester.firstWidget(find.byType(Container));
    final boxDecoration = container.decoration as BoxDecoration;
    final gradient = boxDecoration.gradient as LinearGradient;
    expect(gradient.colors, equals([Colors.blueGrey, Colors.blueGrey]));
  });
}

MultiProvider _wrapper({
  required child,
  Config? cfg,
  Urls? urls,
  Events? ev,
  DocsService? docs,
  DocsState? docsSt,
  ConnService? conn,
  ConnState? connSt,
}) {
  final config = cfg ?? _ConfigMock();
  final urlsProvider = urls ?? Urls(config: config);
  final events = ev ?? Events(urlsProvider: urlsProvider);
  final docsService = docs ?? DocsService(urls: urlsProvider, ev: events);
  final connService = conn ?? ConnService(ev: events);

  return MultiProvider(
    providers: [
      ChangeNotifierProvider<DocsState>(
        create: (_) => DocsState(docsService: docsService),
      ),
      ChangeNotifierProvider<ConnState>(
        create: (_) => ConnState(connService: connService),
      ),
    ],
    child: child,
  );
}

class _ConfigMock extends Config {
  @override
  String get baseUrl => 'http://192.168.16.247:8000';

  @override
  String get websocketUrl => 'ws://192.168.16.247:8001';
}
