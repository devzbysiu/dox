class Document {
  final String filename;

  const Document({required this.filename});

  factory Document.fromJson(Map<String, dynamic> map) {
    return Document(
      filename: map['filename'] ?? '',
    );
  }
}
