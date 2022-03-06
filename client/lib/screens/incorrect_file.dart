import 'package:flutter/material.dart';
import 'package:lottie/lottie.dart';

class IncorrectFileScreen extends StatelessWidget {
  const IncorrectFileScreen({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Lottie.asset(
        'assets/incorrect-file.json',
        height: MediaQuery.of(context).size.height * 1,
        animate: true,
        repeat: true,
      ),
    );
  }
}
