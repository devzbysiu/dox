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
    final connState = ConnStateMock();
    const statusDot = StatusDot();

    // when
    await tester.pumpWidget(wrapper(widget: statusDot, connSt: connState));
    expect(statusDot.color(tester), equals(disconnectedColor()));

    connState.isConnected = true;
    await tester.pump();

    // then
    expect(statusDot.color(tester), equals(connectedColor()));
  }, skip: true); // TODO: stream implementation needs to be properly mocked first

  testWidgets('StatusDot changes color when disconnected', (tester) async {
    // given
    final connState = ConnStateMock();
    const statusDot = StatusDot();

    await tester.pumpWidget(wrapper(widget: statusDot, connSt: connState));
    expect(statusDot.color(tester), equals(disconnectedColor()));

    connState.isConnected = true;
    await tester.pump();
    expect(statusDot.color(tester), equals(connectedColor()));

    // when
    connState.isConnected = false;
    await tester.pump();

    // then
    expect(statusDot.color(tester), equals(disconnectedColor()));
  }, skip: true); // TODO: stream implementation needs to be properly mocked first
}
