import 'package:dox/models/docs_state.dart';
import 'package:dox/utilities/log.dart';
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

class _SearchInputState extends State<SearchInput> with Log {
  final TextEditingController _controller = TextEditingController();

  @override
  Widget build(BuildContext context) {
    final onChanged = context.read<DocsState>().onQueryChanged;
    return Material(
      borderRadius: const BorderRadius.all(Radius.circular(15)),
      elevation: 18,
      shadowColor: context.onBackground,
      child: TextField(
        controller: _controller,
        onChanged: onChanged,
        decoration: _inputDecoration(context),
      ),
    );
  }

  InputDecoration _inputDecoration(BuildContext context) {
    return InputDecoration(
      filled: true,
      fillColor: context.onPrimary,
      hintText: "Search",
      prefixIcon: const Icon(Icons.search),
      suffixIcon: IconButton(
        icon: const Icon(Icons.clear),
        onPressed: () => _clear(context),
      ),
      focusedBorder: _border(),
      enabledBorder: _border(),
      border: _border(),
    );
  }

  void _clear(BuildContext context) async {
    log.fine('clearing input');
    _controller.clear();
    await context.read<DocsState>().reset();
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
