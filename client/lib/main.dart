import 'package:client/document.dart';
import 'package:client/image.dart';
import 'package:client/search_model.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersive);
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: ChangeNotifierProvider(
        create: (_) => SearchModel(),
        child: const MyHomePage(),
      ),
    );
  }
}

class MyHomePage extends StatelessWidget {
  const MyHomePage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Consumer<SearchModel>(
      builder: (context, model, _) => Column(
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              onChanged: (query) async => model.onQueryChanged(query),
              decoration: const InputDecoration(
                  labelText: "Search",
                  hintText: "Search",
                  prefixIcon: Icon(Icons.search),
                  border: OutlineInputBorder(
                      borderRadius: BorderRadius.all(Radius.circular(25.0)))),
            ),
          ),
          Expanded(
            child: ListView(children: buildChildren(model)),
          ),
        ],
      ),
    ));
  }

  List<Widget> buildChildren(SearchModel model) {
    return model.suggestions.map(toImageUrl).map(buildImage).toList();
  }

  String toImageUrl(Document doc) {
    return "http://10.0.2.2:8000/document/${doc.filename}";
  }

  Widget buildImage(String url) {
    return Padding(
        padding: const EdgeInsets.all(15), child: OpenableImage(url: url));
  }
}
