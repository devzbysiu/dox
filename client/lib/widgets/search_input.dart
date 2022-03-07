import 'package:dox/models/docs_model.dart';
import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class SearchInput extends StatefulWidget {
  const SearchInput({
    Key? key,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _SearchInputState();
}

class _SearchInputState extends State<SearchInput> {
  final TextEditingController _controller = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Material(
      borderRadius: const BorderRadius.all(Radius.circular(15)),
      elevation: 18,
      shadowColor: onBackground(context),
      child: Consumer<DocsModel>(
        builder: (context, model, _) => TextField(
          controller: _controller,
          onChanged: model.onQueryChanged,
          decoration: _inputDecoration(context, model),
        ),
      ),
    );
  }

  InputDecoration _inputDecoration(BuildContext context, DocsModel model) {
    return InputDecoration(
      filled: true,
      fillColor: onPrimary(context),
      hintText: "Search",
      prefixIcon: const Icon(Icons.search),
      suffixIcon: IconButton(
        icon: const Icon(Icons.clear),
        onPressed: () => _clear(model),
      ),
      focusedBorder: _border(),
      enabledBorder: _border(),
      border: _border(),
    );
  }

  void _clear(DocsModel model) async {
    _controller.clear();
    await model.reset();
    setState(() {});
  }

  OutlineInputBorder _border() {
    return const OutlineInputBorder(
      borderSide: BorderSide(color: Colors.transparent, width: 0),
      borderRadius: BorderRadius.all(
        Radius.circular(15.0),
      ),
    );
  }
}
