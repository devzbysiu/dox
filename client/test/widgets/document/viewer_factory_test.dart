import 'package:dox/screens/incorrect_file.dart';
import 'package:dox/widgets/document/image_viewer.dart';
import 'package:dox/widgets/document/pdf_viewer.dart';
import 'package:dox/widgets/document/viewer_factory.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../utils.dart';

void main() {
  testWidgets('It returns ImageViewer for images', (tester) async {
    // given
    final signInService = SignInServiceDummy();
    final imageUri = Uri(scheme: 'https', host: 'some-host', path: 'image.png');
    final widget = ViewerFactory.from(imageUri, signInService: signInService);

    // when
    await tester.pumpWidget(await wrap(widget: widget));

    // then
    expect(find.byType(ImageViewer), findsOneWidget);
  });

  testWidgets('It returns PdfViewer for pdfs', (tester) async {
    // given
    final signInService = SignInServiceDummy();
    final pdfUri = Uri(scheme: 'https', host: 'some-host', path: 'file.pdf');
    final widget = ViewerFactory.from(pdfUri, signInService: signInService);

    // when
    await tester.pumpWidget(await wrap(widget: widget));

    // then
    expect(find.byType(PdfViewer), findsOneWidget);
  });

  testWidgets('It returns IncorrectFileScreen for wrong files', (tester) async {
    // given
    final signInService = SignInServiceDummy();
    final badUri = Uri(scheme: 'https', host: 'some-host', path: 'file.docx');
    final widget = ViewerFactory.from(badUri, signInService: signInService);

    // when
    await tester.pumpWidget(await wrap(widget: widget));

    // then
    expect(find.byType(IncorrectFileScreen), findsOneWidget);
  });
}
