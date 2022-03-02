class Document {
  final String filename;
  final String thumbnail;

  const Document({required this.filename, required this.thumbnail});

  factory Document.fromJson(Map<String, dynamic> map) {
    return Document(
      filename: map['filename'] ?? '',
      thumbnail: map['thumbnail'] ?? '',
    );
  }
}
