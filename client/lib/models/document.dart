class Document {
  late final Uri filename;
  late final Uri thumbnail;

  Document(Map<String, dynamic> map) {
    filename = map['filename'];
    thumbnail = map['thumbnail'];
  }
}
