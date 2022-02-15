import 'package:dox/models/search_model.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:dox/widgets/app_bar.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:dox/widgets/scan_button.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class HomePage extends StatelessWidget {
  late final DoxService _dox;

  HomePage(DoxService dox, {Key? key}) : super(key: key) {
    _dox = dox;
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
        onTap: () => _hideKeyboard(),
        onVerticalDragDown: (_) => _hideKeyboard(),
        child: Consumer<SearchModel>(
            builder: (context, model, _) => Scaffold(
                backgroundColor: Colors.white,
                body: NestedScrollView(
                  headerSliverBuilder: _scrollableAppBarBuilder,
                  body: _searchInput(model),
                ),
                floatingActionButton:
                    ScanButton(_dox, onScanned: model.clear))));
  }

  void _hideKeyboard() {
    FocusManager.instance.primaryFocus?.unfocus();
  }

  List<Widget> _scrollableAppBarBuilder(BuildContext _ctx, bool _) {
    return const [ScrollableAppBar()];
  }

  Widget _searchInput(SearchModel model) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: SearchInput(onChanged: model.onQueryChanged),
        ),
        Expanded(child: OpenableImageList(urls: model.docUrls)),
      ],
    );
  }
}
