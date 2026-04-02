import 'package:flutter_test/flutter_test.dart';

import 'package:mykey/main.dart';

void main() {
  testWidgets('renders MyKey UI', (WidgetTester tester) async {
    await tester.pumpWidget(const MyKeyApp());

    expect(find.text('MyKey'), findsOneWidget);
    expect(find.text('主密码'), findsOneWidget);
    expect(find.text('服务标识'), findsOneWidget);
    expect(find.text('计算结果'), findsOneWidget);
    expect(find.text('计算密码'), findsOneWidget);
    expect(find.text('复制密码'), findsOneWidget);
    expect(find.text('准备就绪'), findsOneWidget);
  });
}
