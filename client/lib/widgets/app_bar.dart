import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';

class ScrollableAppBar extends StatelessWidget {
  const ScrollableAppBar({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverAppBar(
      title: Container(
        width: 15,
        height: 15,
        decoration: BoxDecoration(
            shape: BoxShape.circle,
            gradient: LinearGradient(
                begin: Alignment.topRight,
                end: Alignment.bottomLeft,
                colors: [Colors.green[300]!, Colors.yellow[400]!])),
      ),
      expandedHeight: 220.0,
      flexibleSpace: FlexibleSpaceBar(
        background: Image.asset('assets/app-bar.webp', fit: BoxFit.cover),
      ),
    );
  }
}
