import 'package:dox/models/connection_state.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/utilities/api.dart';
import 'package:dox/utilities/theme.dart';
import 'package:dox/widgets/add_button.dart';
import 'package:dox/widgets/app_bar.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class HomePage extends StatelessWidget {
  const HomePage({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider<DocsState>(create: (_) => DocsState(Api())),
        ChangeNotifierProvider<ConnState>(create: (_) => ConnState(Api()))
      ],
      child: GestureDetector(
        onTap: () => _hideKeyboard(),
        onVerticalDragDown: (_) => _hideKeyboard(),
        child: Consumer<DocsState>(
          builder: (context, model, _) => Scaffold(
            backgroundColor: background(context),
            body: NestedScrollView(
              headerSliverBuilder: _scrollableAppBarBuilder,
              body: Column(
                children: [
                  const Padding(
                    padding: EdgeInsets.all(8.0),
                    child: SearchInput(),
                  ),
                  Expanded(
                    child: RefreshIndicator(
                      child: const OpenableImageList(),
                      onRefresh: () => _refreshDocs(model),
                    ),
                  ),
                ],
              ),
            ),
            floatingActionButton: const AddButton(),
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

  Future<void> _refreshDocs(DocsState model) async {
    await Future.delayed(const Duration(seconds: 1), model.refresh);
  }
}
