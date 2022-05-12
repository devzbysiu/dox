import 'package:dox/utilities/connection.dart';
import 'package:dox/utilities/log.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class StatusDot extends StatefulWidget {
  const StatusDot({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _StatusDotState();
}

class _StatusDotState extends State<StatusDot> with Log {
  @override
  Widget build(BuildContext context) {
    final connection = context.watch<Connection>();
    connection.onConnected(turnOn);
    connection.onDisconnected(turnOff);

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

  void turnOn() {
    setState(() {
      _dotColors = [Colors.green[300]!, Colors.yellow[400]!];
    });
  }

  void turnOff() {
    setState(() {
      _dotColors = [Colors.blueGrey, Colors.blueGrey];
    });
  }

  List<Color> _dotColors = [Colors.blueGrey, Colors.blueGrey];
}
