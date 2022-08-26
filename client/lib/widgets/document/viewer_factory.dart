import 'package:dox/screens/incorrect_file.dart';
import 'package:dox/services/sign_in_service.dart';
import 'package:dox/utilities/filetype.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/widgets/document/image_viewer.dart';
import 'package:dox/widgets/document/pdf_viewer.dart';
import 'package:flutter/material.dart';

class ViewerFactory {
  ViewerFactory._();

  static Widget from(Uri uri, {SignInService? signInService}) {
    signInService ??= getIt<SignInService>();
    final authHeaders = signInService.authHeaders;
    switch (uri.filetype()) {
      case Filetype.image:
        return ImageViewer(fileUrl: uri, headers: authHeaders);
      case Filetype.pdf:
        return PdfViewer(fileUrl: uri, headers: authHeaders);
      default:
        return const IncorrectFileScreen();
    }
  }
}
