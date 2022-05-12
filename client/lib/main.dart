import 'package:dox/dox.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

Config? configOverride;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
  await setupServices(configOverride: configOverride);
  runApp(const Dox());
}
