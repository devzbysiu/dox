import 'package:flutter/material.dart';

class SearchInput extends StatelessWidget {
  final Function(String) onQueryChanged;

  const SearchInput({Key? key, required this.onQueryChanged}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Material(
        borderRadius: const BorderRadius.all(Radius.circular(15)),
        elevation: 18,
        shadowColor: Colors.black,
        child: TextField(
            onChanged: onQueryChanged, decoration: _inputDecoration()));
  }

  InputDecoration _inputDecoration() {
    return InputDecoration(
        filled: true,
        fillColor: Colors.white,
        hintText: "Search",
        prefixIcon: const Icon(Icons.search),
        focusedBorder: _border(),
        enabledBorder: _border(),
        border: _border());
  }

  OutlineInputBorder _border() {
    return const OutlineInputBorder(
        borderSide: BorderSide(color: Colors.transparent, width: 0),
        borderRadius: BorderRadius.all(Radius.circular(15.0)));
  }
}
