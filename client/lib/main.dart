import 'package:dox/models/search_model.dart';
import 'package:dox/screens/home_page.dart';
import 'package:dox/utilities/config.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
  runApp(MyApp(await Config.init()));
}

class MyApp extends StatelessWidget {
  late final Config _config;

  MyApp(Config config, {Key? key}) : super(key: key) {
    _config = config;
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: ChangeNotifierProvider(
        create: (_) => SearchModel(_config),
        child: const HomePage(),
      ),
    );
  }
}
