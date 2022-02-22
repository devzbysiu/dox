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

Color primary(BuildContext context) {
  return Theme.of(context).colorScheme.primary;
}

Color secondary(BuildContext context) {
  return Theme.of(context).colorScheme.secondary;
}

Color onPrimary(BuildContext context) {
  return Theme.of(context).colorScheme.onPrimary;
}

Color background(BuildContext context) {
  return Theme.of(context).colorScheme.background;
}

Color onBackground(BuildContext context) {
  return Theme.of(context).colorScheme.onBackground;
}
