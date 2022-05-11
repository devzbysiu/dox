import 'package:dox/models/connection_state.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/screens/home_page.dart';
import 'package:dox/services/lifecycle_service.dart';
import 'package:dox/utilities/notifications_stream.dart';
import 'package:flutter/material.dart';
import 'package:lottie/lottie.dart';
import 'package:provider/provider.dart';

class SplashScreen extends StatefulWidget {
  const SplashScreen({
    Key? key,
  }) : super(key: key);

  @override
  _SplashScreenState createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen>
    with TickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(seconds: (5)),
      vsync: this,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Lottie.asset(
        'assets/splash-screen.json',
        controller: _controller,
        height: MediaQuery.of(context).size.height * 1,
        animate: true,
        onLoaded: (composition) {
          _controller
            ..duration = composition.duration
            ..forward().whenComplete(
              () => Navigator.pushReplacement(
                context,
                MaterialPageRoute(
                    builder: (context) => MultiProvider(providers: [
                          ChangeNotifierProvider<DocsState>(
                              create: (_) => DocsStateImpl()),
                          ChangeNotifierProvider<ConnState>(
                              create: (_) => ConnStateImpl()),
                          ChangeNotifierProvider<NotificationsStream>(
                              create: (_) => NotificationsStreamImpl()),
                        ], builder: (context, _) => const HomePage())),
              ),
            );
        },
      ),
    );
  }
}
