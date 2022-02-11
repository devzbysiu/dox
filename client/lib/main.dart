import 'package:dox/models/search_model.dart';
import 'package:dox/screens/home_page.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
  runApp(MyApp(await Config.init()));
}

class MyApp extends StatelessWidget {
  late final Urls _urls;

  MyApp(Config config, {Key? key}) : super(key: key) {
    _urls = Urls(config);
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Dox',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: ChangeNotifierProvider(
        create: (_) => SearchModel(_urls),
        child: HomePage(_urls),
      ),
    );
  }
}
