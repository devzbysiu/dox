Uri searchEndpoint(String query) {
  return Uri.parse('http://10.0.2.2:8000/search?q=$query');
}

Uri allDocumentsEndpoint() {
  return Uri.parse('http://10.0.2.2:8000/documents/all');
}