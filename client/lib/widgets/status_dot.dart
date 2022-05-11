import 'package:dox/models/connection_state.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/notifications_stream.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class StatusDot extends StatefulWidget {
  const StatusDot({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _StatusDotState();
}

class _StatusDotState extends State<StatusDot> with Log {
  _StatusDotState() {
    _dotColors = [Colors.blueGrey, Colors.blueGrey];
  }

  @override
  Widget build(BuildContext context) {
    final stream = context.select((NotificationsStream notificationsStream) =>
        notificationsStream.stream);
    stream.listen((data) {
      if (data == 'connected') {
        log.fine('connected event received, changing colors');
        setState(() {
          _dotColors = [Colors.green[300]!, Colors.yellow[400]!];
        });
      }
    }, onDone: () {
      setState(() {
        _dotColors = [Colors.blueGrey, Colors.blueGrey];
      });
    });
    return Container(
      width: 15,
      height: 15,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        gradient: LinearGradient(
          begin: Alignment.topRight,
          end: Alignment.bottomLeft,
          colors: _dotColors,
        ),
      ),
    );
  }

  List<Color> _colors(BuildContext context) {
    final isConnected = context.select((ConnState state) => state.isConnected);
    log.fine('showing status color for: "isConnected == $isConnected"');
    return isConnected
        ? [Colors.green[300]!, Colors.yellow[400]!]
        : [Colors.blueGrey, Colors.blueGrey];
  }

  late List<Color> _dotColors;
}
