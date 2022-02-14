import 'package:flutter/material.dart';

class SearchInput extends StatefulWidget {
  final Function(String) onQueryChanged;

  const SearchInput({Key? key, required this.onQueryChanged}) : super(key: key);

  @override
  State<StatefulWidget> createState() =>
      _SearchInputState(onQueryChanged: onQueryChanged);
}

class _SearchInputState extends State<SearchInput> {
  final Function(String) onQueryChanged;

  final TextEditingController _controller = TextEditingController();

  _SearchInputState({required this.onQueryChanged});

  @override
  Widget build(BuildContext context) {
    return Material(
        borderRadius: const BorderRadius.all(Radius.circular(15)),
        elevation: 18,
        shadowColor: Colors.black,
        child: TextField(
            controller: _controller,
            onChanged: onQueryChanged,
            decoration: _inputDecoration()));
  }

  InputDecoration _inputDecoration() {
    return InputDecoration(
        filled: true,
        fillColor: Colors.white,
        hintText: "Search",
        prefixIcon: const Icon(Icons.search),
        suffixIcon:
            IconButton(icon: const Icon(Icons.clear), onPressed: _clear),
        focusedBorder: _border(),
        enabledBorder: _border(),
        border: _border());
  }

  void _clear() {
    _controller.clear();
    onQueryChanged('');
    setState(() {});
  }

  OutlineInputBorder _border() {
    return const OutlineInputBorder(
        borderSide: BorderSide(color: Colors.transparent, width: 0),
        borderRadius: BorderRadius.all(Radius.circular(15.0)));
  }
}
