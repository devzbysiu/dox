import 'package:dox/widgets/status_dot_2.dart';
import 'package:flutter/material.dart';

class ScrollableAppBar extends StatelessWidget {
  const ScrollableAppBar({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverAppBar(
      title: const StatusDot(),
      expandedHeight: 220.0,
      flexibleSpace: FlexibleSpaceBar(
        background: Image.asset('assets/app-bar.webp', fit: BoxFit.cover),
      ),
    );
  }
}
