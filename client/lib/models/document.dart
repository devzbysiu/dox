import 'package:dox/utilities/filetype.dart';

class Document {
  const Document(this.fileUrl, this.thumbnailUrl);

  final Uri fileUrl;

  final Uri thumbnailUrl;

  bool isSupported() {
    final docType = fileUrl.filetype();
    final thumbnailType = thumbnailUrl.filetype();
    return (docType.isImage || docType.isPdf) && thumbnailType.isImage;
  }
}
