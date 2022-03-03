class Document {
  late final Uri fileUrl;
  late final Uri thumbnailUrl;

  Document(Map<String, dynamic> map) {
    fileUrl = map['fileUrl'];
    thumbnailUrl = map['thumbnailUrl'];
  }
}
