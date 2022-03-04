import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';

class ScrollableAppBar extends StatelessWidget {
  const ScrollableAppBar({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverAppBar(
      title: Text('Dox', style: TextStyle(color: onPrimary(context))),
      expandedHeight: 220.0,
      flexibleSpace: FlexibleSpaceBar(
        centerTitle: true,
        background: Image.asset('assets/app-bar.webp', fit: BoxFit.cover),
      ),
    );
  }
}
