import 'package:client/document.dart';
import 'package:client/search_model.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:photo_view/photo_view.dart';
import 'package:provider/provider.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.leanBack);
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: ChangeNotifierProvider(
        create: (_) => SearchModel(),
        child: const MyHomePage(title: 'ListView with Search'),
      ),
    );
  }
}

class MyHomePage extends StatefulWidget {
  final String title;

  const MyHomePage({Key? key, required this.title}) : super(key: key);

  @override
  _MyHomePageState createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  TextEditingController editingController = TextEditingController();

  final duplicateItems = List<String>.generate(10000, (i) => "Item $i");
  var items = <String>[];

  @override
  void initState() {
    items.addAll(duplicateItems);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Consumer<SearchModel>(
      builder: (context, model, _) => Column(
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              onChanged: (value) async {
                await filterSearchResults(model, value);
              },
              controller: editingController,
              decoration: const InputDecoration(
                  labelText: "Search",
                  hintText: "Search",
                  prefixIcon: Icon(Icons.search),
                  border: OutlineInputBorder(
                      borderRadius: BorderRadius.all(Radius.circular(25.0)))),
            ),
          ),
          Expanded(
            child: buildListView(model),
          ),
        ],
      ),
    ));
  }

  Future<void> filterSearchResults(SearchModel model, String query) async {
    model.onQueryChanged(query);
  }

  Widget buildListView(SearchModel model) {
    final children = model.suggestions
        .map(toImageUrl)
        .map((url) => Padding(
            padding: const EdgeInsets.all(15), child: OpenableImage(url: url)))
        .toList();
    return ListView(children: children);
  }

  String toImageUrl(Document doc) {
    return "http://10.0.2.2:8000/document/${doc.filename}";
  }
}

class OpenableImage extends StatelessWidget {
  final String url;

  const OpenableImage({required this.url});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: GestureDetector(
        onTap: () {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (context) => HeroPhotoViewRouteWrapper(
                imageProvider: NetworkImage(url),
              ),
            ),
          );
        },
        child: Hero(
          tag: "someTag",
          child: Image.network(
            url,
            width: 350.0,
            loadingBuilder: (_, child, chunk) =>
                chunk != null ? const Text("loading") : child,
          ),
        ),
      ),
    );
  }
}

class HeroPhotoViewRouteWrapper extends StatelessWidget {
  final ImageProvider imageProvider;
  final BoxDecoration? backgroundDecoration;

  const HeroPhotoViewRouteWrapper({
    required this.imageProvider,
    this.backgroundDecoration,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: BoxConstraints.expand(
        height: MediaQuery.of(context).size.height,
      ),
      child: PhotoView(
        imageProvider: imageProvider,
        backgroundDecoration: backgroundDecoration,
      ),
    );
  }
}
