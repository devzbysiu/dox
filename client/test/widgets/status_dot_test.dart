import 'package:dox/widgets/status_dot.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../utils.dart';

void main() {
  testWidgets('StatusDot initially displays gray dot', (tester) async {
    // given
    const statusDot = StatusDot(key: Key('StatusDot'));

    // when
    await tester.pumpWidget(wrapper(widget: statusDot));

    // then
    expect(statusDot.color(tester), equals([Colors.blueGrey, Colors.blueGrey]));
  });

  testWidgets('StatusDot changes color when connected', (tester) async {
    // given
    final connState = ConnStateMock();
    const statusDot = StatusDot();

    // when
    await tester.pumpWidget(wrapper(widget: statusDot, connSt: connState));
    expect(statusDot.color(tester), equals([Colors.blueGrey, Colors.blueGrey]));

    connState.isConnected = true;
    await tester.pump();

    // then
    expect(
      statusDot.color(tester),
      equals([Colors.green[300]!, Colors.yellow[400]!]),
    );
  });
}