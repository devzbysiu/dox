import 'package:dox/widgets/document/image_viewer.dart';
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
}
