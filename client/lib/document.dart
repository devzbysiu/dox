class Document {
  final String filename;

  const Document({required this.filename});

  factory Document.fromJson(Map<String, dynamic> map) {
    return Document(
      filename: map['filename'] ?? '',
    );
  }

  @override
  String toString() => 'Document(filename: $filename)';

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;

    return other is Document && other.filename == filename;
  }

  @override
  int get hashCode => filename.hashCode;
}
