import 'package:dox/models/app_state.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class StatusDot extends StatelessWidget {
  const StatusDot({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Consumer<AppState>(
      builder: (context, model, _) => Container(
        width: 15,
        height: 15,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          gradient: LinearGradient(
            begin: Alignment.topRight,
            end: Alignment.bottomLeft,
            colors: _colors(model),
          ),
        ),
      ),
    );
  }

  List<Color> _colors(AppState model) {
    return model.isConnected
        ? [Colors.green[300]!, Colors.yellow[400]!]
        : [Colors.blueGrey, Colors.blueGrey];
  }
}
