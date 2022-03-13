import 'package:dox/screens/splash_screen.dart';
import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';

class Dox extends StatelessWidget {
  const Dox({
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
