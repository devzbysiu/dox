import 'package:dox/screens/splash_screen.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
  await setupServices();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({
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
