import 'package:dox/models/search_model.dart';
import 'package:dox/utilities/api.dart';
import 'package:dox/utilities/theme.dart';
import 'package:dox/widgets/app_bar.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:dox/widgets/add_button.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class HomePage extends StatelessWidget {
  const HomePage({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (_) => SearchModel(Api()),
      child: GestureDetector(
        onTap: () => _hideKeyboard(),
        onVerticalDragDown: (_) => _hideKeyboard(),
        child: Consumer<SearchModel>(
          builder: (context, model, _) => Scaffold(
            backgroundColor: background(context),
            body: NestedScrollView(
              headerSliverBuilder: _scrollableAppBarBuilder,
              body: _searchInput(model),
            ),
            floatingActionButton: AddButton(model),
          ),
        ),
      ),
    );
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
        Expanded(
          child: RefreshIndicator(
            child: OpenableImageList(docUrls: model.suggestions),
            onRefresh: () => _refreshDocs(model),
          ),
        ),
      ],
    );
  }

  Future<void> _refreshDocs(SearchModel model) async {
    await Future.delayed(const Duration(seconds: 1), model.refresh);
  }
}
