import 'package:dox/utilities/log.dart';
import 'package:flutter/cupertino.dart';

class Lifecycle extends StatefulWidget {
  final Widget child;

  const Lifecycle({Key? key, required this.child}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _LifecycleState();
}

class _LifecycleState extends State<Lifecycle> with WidgetsBindingObserver, Log {
  @override
  Widget build(BuildContext context) {
    return widget.child;
  }

  @override
  void initState() {
    WidgetsBinding.instance.addObserver(this);
    super.initState();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.paused:
        log.fine('app paused');
        break;
      case AppLifecycleState.resumed:
        log.fine('app resumed');
        break;
      default:
        break;
    }
  }
}