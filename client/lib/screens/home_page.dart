import 'package:dox/models/search_model.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class HomePage extends StatelessWidget {
  const HomePage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
        onTap: () => _hideKeyboard(),
        onVerticalDragDown: (_) => _hideKeyboard(),
        child: Scaffold(
            body: Consumer<SearchModel>(
          builder: (context, model, _) => Column(
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: SearchInput(onQueryChanged: model.onQueryChanged),
              ),
              Expanded(child: OpenableImageList(docUrls: model.docUrls)),
            ],
          ),
        )));
  }

  void _hideKeyboard() {
    FocusManager.instance.primaryFocus?.unfocus();
  }
}
