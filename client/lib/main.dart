import 'package:dox/screens/splash_screen.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/dox_service.dart';
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
  late final DoxService _dox;

  MyApp(Config config, {Key? key}) : super(key: key) {
    _dox = DoxService(Urls(config));
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Dox',
      theme: theme(),
      home: SplashScreen(_dox),
    );
  }
}
