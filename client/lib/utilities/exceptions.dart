class ApiNotInitializedException implements Exception {
  @override
  String toString() => 'You need to initialize it first';
}

class FiletypeNotSupportedException implements Exception {
  final Uri _uri;

  FiletypeNotSupportedException(this._uri);

  @override
  String toString() => 'Filetype not supported for: "$_uri"';
}
