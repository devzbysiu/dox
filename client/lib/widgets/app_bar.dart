import 'package:flutter/material.dart';

class ScrollableAppBar extends StatelessWidget {
  const ScrollableAppBar({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverAppBar(
      title: const Text('Dox', style: TextStyle(color: Colors.white)),
      expandedHeight: 220.0,
      flexibleSpace: FlexibleSpaceBar(
          centerTitle: true,
          background: Image.asset('assets/app-bar.webp', fit: BoxFit.cover)),
    );
  }
}
