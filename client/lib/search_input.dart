import 'package:flutter/material.dart';

class SearchInput extends StatelessWidget {
  final Function(String) onQueryChanged;

  const SearchInput({Key? key, required this.onQueryChanged}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return TextField(
        onChanged: onQueryChanged,
        decoration: const InputDecoration(
            labelText: "Search",
            hintText: "Search",
            prefixIcon: Icon(Icons.search),
            border: OutlineInputBorder(
                borderRadius: BorderRadius.all(Radius.circular(25.0)))));
  }
}
