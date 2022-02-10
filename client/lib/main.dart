import 'package:dox/image.dart';
import 'package:dox/search_input.dart';
import 'package:dox/search_model.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import 'config.dart';

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
        child: const MyHomePage(),
      ),
    );
  }
}

class MyHomePage extends StatelessWidget {
  const MyHomePage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
        onTap: () => FocusManager.instance.primaryFocus?.unfocus(),
        onVerticalDragDown: (_) =>
            FocusManager.instance.primaryFocus?.unfocus(),
        child: Scaffold(
            body: Consumer<SearchModel>(
          builder: (context, model, _) => Column(
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: SearchInput(onQueryChanged: model.onQueryChanged),
              ),
              Expanded(
                child: ListView(children: buildChildren(model)),
              ),
            ],
          ),
        )));
  }

  List<Widget> buildChildren(SearchModel model) {
    return model.docUrls.map(buildImage).toList();
  }

  Widget buildImage(Uri url) {
    return Padding(
        padding: const EdgeInsets.all(15), child: OpenableImage(url: url));
  }
}
