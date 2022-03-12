import 'package:dox/screens/splash_screen.dart';
import 'package:dox/services/connection_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/theme.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
  runApp(MyApp(await Config.init()));
}

class MyApp extends StatelessWidget {
  MyApp(
    Config config, {
    Key? key,
  }) : super(key: key) {
    final urls = Urls(config);
    final eventsStream = EventsStream(urls.notifications());
    DocsService.init(urls, eventsStream);
    ConnService.init(eventsStream);
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Dox',
      theme: theme(),
      home: const SplashScreen(),
    );
  }
}
