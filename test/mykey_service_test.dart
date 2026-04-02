import 'package:flutter_test/flutter_test.dart';
import 'package:mykey/src/mykey_service.dart';

void main() {
  const service = MyKeyService();

  test('rejects empty master password', () async {
    expect(
      () => service.derivePassword(masterPassword: '', service: 'github.com'),
      throwsA(isA<MyKeyException>()),
    );
  });

  test('rejects empty service identifier', () async {
    expect(
      () => service.derivePassword(masterPassword: 'secret', service: ''),
      throwsA(isA<MyKeyException>()),
    );
  });

  test('matches gokey output for github.com', () async {
    final password = await service.derivePassword(
      masterPassword: 'correct horse battery staple',
      service: 'github.com',
    );

    expect(password, 'jjbHr0a6*K_^-KsK');
  });

  test('matches gokey output for gmail.com', () async {
    final password = await service.derivePassword(
      masterPassword: 'correct horse battery staple',
      service: 'gmail.com',
    );

    expect(password, '_Cjw**lIR5lrB^1X');
  });

  test('matches gokey output for example.com', () async {
    final password = await service.derivePassword(
      masterPassword: 'secret',
      service: 'example.com',
    );

    expect(password, 'pR2!h3Ap3z)hMOL!');
  });
}
