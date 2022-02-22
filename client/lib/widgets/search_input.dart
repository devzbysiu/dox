import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';

class SearchInput extends StatefulWidget {
  final Function(String) onChanged;

  const SearchInput({Key? key, required this.onChanged}) : super(key: key);

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
      child: TextField(
        controller: _controller,
        onChanged: widget.onChanged,
        decoration: _inputDecoration(context),
      ),
    );
  }

  InputDecoration _inputDecoration(BuildContext context) {
    return InputDecoration(
      filled: true,
      fillColor: onPrimary(context),
      hintText: "Search",
      prefixIcon: const Icon(Icons.search),
      suffixIcon: IconButton(icon: const Icon(Icons.clear), onPressed: _clear),
      focusedBorder: _border(),
      enabledBorder: _border(),
      border: _border(),
    );
  }

  void _clear() {
    _controller.clear();
    widget.onChanged('');
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
