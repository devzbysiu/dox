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
    // given
    const statusDot = StatusDot();

    // when
    await tester.pumpWidget(_wrapper(child: statusDot));

    // then
    expect(statusDot.color(tester), equals([Colors.blueGrey, Colors.blueGrey]));
  });

  // testWidgets("StatusDot changes color when connected", (tester) async {
  //   // given
  //   final connState = _ConnStateMock();
  //   const statusDot = StatusDot();
  //
  //   // when
  //   await tester.pumpWidget(_wrapper(child: statusDot, connSt: connState));
  //   Container container = tester.firstWidget(find.byType(Container));
  //   var boxDecoration = container.decoration as BoxDecoration;
  //   var gradient = boxDecoration.gradient as LinearGradient;
  //   expect(gradient.colors, equals([Colors.blueGrey, Colors.blueGrey]));
  //
  //   connState.isConnected = true;
  //   await tester.pump();
  //
  //   // then
  //   container = tester.firstWidget(find.byType(Container));
  //   boxDecoration = container.decoration as BoxDecoration;
  //   gradient = boxDecoration.gradient as LinearGradient;
  //   expect(gradient.colors, equals([Colors.green[300]!, Colors.yellow[400]!]));
  // });
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
  DocsState docsState(_) => docsSt ?? DocsState(docsService: docsService);
  ConnState connState(_) => connSt ?? ConnStateImpl(connService: connService);

  return MultiProvider(
    providers: [
      ChangeNotifierProvider<DocsState>(create: docsState),
      ChangeNotifierProvider<ConnState>(create: connState),
    ],
    child: child,
  );
}

class _ConfigMock implements Config {
  @override
  String get baseUrl => 'http://192.168.16.247:8000';

  @override
  String get websocketUrl => 'ws://192.168.16.247:8001';
}

class _ConnStateMock extends ChangeNotifier implements ConnState {
  bool _isConnected = false;

  @override
  bool get isConnected => false;

  set isConnected(val) {
    _isConnected = val;
    notifyListeners();
  }
}

extension StatusDotExt on StatusDot {
  List<Color> color(WidgetTester tester) {
    final Container container = tester.firstWidget(find.byType(Container));
    final boxDecoration = container.decoration as BoxDecoration;
    final gradient = boxDecoration.gradient as LinearGradient;
    return gradient.colors;
  }
}
