import 'package:dox/models/docs_model.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class ScrollableAppBar extends StatelessWidget {
  const ScrollableAppBar({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverAppBar(
      title: Consumer<DocsModel>(
        builder: (context, model, _) => Container(
          width: 15,
          height: 15,
          decoration: BoxDecoration(
              shape: BoxShape.circle,
              gradient: LinearGradient(
                  begin: Alignment.topRight,
                  end: Alignment.bottomLeft,
                  colors: _colors(model))),
        ),
      ),
      expandedHeight: 220.0,
      flexibleSpace: FlexibleSpaceBar(
        background: Image.asset('assets/app-bar.webp', fit: BoxFit.cover),
      ),
    );
  }

  List<Color> _colors(DocsModel model) {
    return model.isConnected
        ? [Colors.green[300]!, Colors.yellow[400]!]
        : [Colors.blueGrey, Colors.blueGrey];
  }
}
