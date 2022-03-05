import 'package:flutter/material.dart';
import 'package:lottie/lottie.dart';

class IncorrectFileScreen extends StatefulWidget {
  const IncorrectFileScreen({
    Key? key,
  }) : super(key: key);

  @override
  _IncorrectFileScreenState createState() => _IncorrectFileScreenState();
}

class _IncorrectFileScreenState extends State<IncorrectFileScreen> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Lottie.asset(
        // TODO: try to find better animation
        'assets/incorrect-file.json',
        height: MediaQuery.of(context).size.height * 1,
        animate: true,
        repeat: true,
      ),
    );
  }
}
