import 'package:dox/screens/splash_screen.dart';
import 'package:dox/services/connection_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/theme.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);

  getIt.registerSingleton<Config>(await Config.init());
  getIt.registerSingleton<Urls>(Urls());
  getIt.registerSingleton<EventsStream>(EventsStream());
  getIt.registerSingleton<DocsService>(DocsService());
  getIt.registerSingleton<ConnService>(ConnService());

  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({
      Config? cfg,
      Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Dox',
      theme: theme(),
      home: const SplashScreen(),
    );
  }
}
