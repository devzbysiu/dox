import 'package:flutter/material.dart';

class ScrollableAppBar extends StatelessWidget {
  const ScrollableAppBar({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverAppBar(
      expandedHeight: 220.0,
      flexibleSpace: FlexibleSpaceBar(
        background: Image.asset('assets/app-bar.webp', fit: BoxFit.cover),
      ),
    );
  }
}
