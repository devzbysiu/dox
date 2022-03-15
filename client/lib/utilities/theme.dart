import 'package:flutter/material.dart';

ThemeData theme() {
  return ThemeData(
    colorScheme: ColorScheme.fromSwatch(
      primarySwatch: Colors.purple,
    ).copyWith(
        onBackground: Colors.black,
        secondary: Colors.deepPurple,
        onPrimary: Colors.white,
        background: Colors.white),
  );
}

extension BuildContextExt on BuildContext {
  Color get primary => Theme.of(this).colorScheme.primary;
  Color get secondary => Theme.of(this).colorScheme.secondary;
  Color get onPrimary => Theme.of(this).colorScheme.onPrimary;
  Color get background => Theme.of(this).colorScheme.background;
  Color get onBackground => Theme.of(this).colorScheme.onBackground;
}
