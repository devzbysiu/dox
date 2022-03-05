import 'package:flutter/material.dart';
import 'package:lottie/lottie.dart';

class IncorrectFile extends StatefulWidget {
  const IncorrectFile({
    Key? key,
  }) : super(key: key);

  @override
  _IncorrectFileState createState() => _IncorrectFileState();
}

class _IncorrectFileState extends State<IncorrectFile>
    with TickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(seconds: 4),
      vsync: this,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Lottie.asset(
        'assets/incorrect-file.json',
        controller: _controller,
        height: MediaQuery.of(context).size.height * 1,
        animate: true,
        onLoaded: (composition) {
          _controller
            ..duration = composition.duration
            ..forward();
        },
      ),
    );
  }
}
