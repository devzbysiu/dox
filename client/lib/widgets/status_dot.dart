import 'package:dox/models/docs_model.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class StatusDot extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Consumer<DocsModel>(
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

  List<Color> _colors(DocsModel model) {
    return model.isConnected
        ? [Colors.green[300]!, Colors.yellow[400]!]
        : [Colors.blueGrey, Colors.blueGrey];
  }
}
