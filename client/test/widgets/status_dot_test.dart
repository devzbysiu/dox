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
    expect(statusDot.color(tester), equals(disconnectedColor()));
  });

  testWidgets('StatusDot changes color when connected', (tester) async {
    // given
    final conn = ConnectionMock();
    const statusDot = StatusDot();

    // when
    await tester.pumpWidget(wrapper(widget: statusDot, conn: conn));
    expect(statusDot.color(tester), equals(disconnectedColor()));

    conn.forceConnected();
    await tester.pump();

    // then
    expect(statusDot.color(tester), equals(connectedColor()));
  }); // TODO: stream implementation needs to be properly mocked first

  testWidgets('StatusDot changes color when disconnected', (tester) async {
    // given
    final conn = ConnectionMock();
    const statusDot = StatusDot();

    await tester.pumpWidget(wrapper(widget: statusDot, conn: conn));
    expect(statusDot.color(tester), equals(disconnectedColor()));

    conn.forceConnected();
    await tester.pump();
    expect(statusDot.color(tester), equals(connectedColor()));

    // when
    conn.forceDisconnected();
    await tester.pump();

    // then
    expect(statusDot.color(tester), equals(disconnectedColor()));
  });
}
