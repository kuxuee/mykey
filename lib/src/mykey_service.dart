import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

class MyKeyException implements Exception {
  const MyKeyException(this.message);

  final String message;

  @override
  String toString() => message;
}

class PasswordSpec {
  const PasswordSpec({
    required this.length,
    required this.upper,
    required this.lower,
    required this.digits,
    required this.special,
    required this.allowedSpecial,
  });

  final int length;
  final int upper;
  final int lower;
  final int digits;
  final int special;
  final String allowedSpecial;

  bool get isValid =>
      length >= upper + lower + digits + special &&
      allowedSpecial.runes.every(_isAllowedSpecialRune);
}

class MyKeyService {
  const MyKeyService();

  static const PasswordSpec defaultSpec = PasswordSpec(
    length: 16,
    upper: 1,
    lower: 1,
    digits: 1,
    special: 1,
    allowedSpecial: '!@#\$%^&*()-_=+?',
  );

  static const String _chars =
      'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`~!@#\$%^&*()-_=+[{]}\\|;:\'",<.>/?';

  Future<String> derivePassword({
    required String masterPassword,
    required String service,
    PasswordSpec spec = defaultSpec,
  }) async {
    if (masterPassword.isEmpty) {
      throw const MyKeyException('主密码不能为空');
    }

    if (service.isEmpty) {
      throw const MyKeyException('服务标识不能为空');
    }

    if (!spec.isValid) {
      throw const MyKeyException('密码规则无效');
    }

    final stream = await _DeterministicByteStream.create(
      password: masterPassword,
      realm: '$service-pass',
    );
    await stream.warmUp(65536);

    while (true) {
      final candidate = _generatePasswordCandidate(stream, spec.length);
      if (_isCompliant(candidate, spec)) {
        return candidate;
      }
    }
  }

  String _generatePasswordCandidate(_DeterministicByteStream stream, int length) {
    final buffer = StringBuffer();

    for (var i = 0; i < length; i++) {
      final value = _randRange(stream, _chars.length);
      buffer.write(_chars[value]);
    }

    return buffer.toString();
  }

  int _randRange(_DeterministicByteStream stream, int max) {
    if (max <= 0 || max >= 255) {
      throw const MyKeyException('字符集长度超出允许范围');
    }

    final remainder = 255 % max;
    final bucket = 255 ~/ max;

    while (true) {
      final value = stream.readByte();
      if (value == 255) {
        continue;
      }

      if (value < 255 - remainder) {
        return value ~/ bucket;
      }
    }
  }

  bool _isCompliant(String password, PasswordSpec spec) {
    var upper = 0;
    var lower = 0;
    var digits = 0;
    var special = 0;

    for (final rune in password.runes) {
      if (_isUpperRune(rune)) {
        upper++;
      }

      if (_isLowerRune(rune)) {
        lower++;
      }

      if (_isDigitRune(rune)) {
        digits++;
      }

      if (_isSpecialRune(rune)) {
        if (spec.allowedSpecial.isEmpty) {
          special++;
        } else if (spec.allowedSpecial.runes.contains(rune)) {
          special++;
        } else {
          return false;
        }
      }
    }

    return _allowed(upper, spec.upper) &&
        _allowed(lower, spec.lower) &&
        _allowed(digits, spec.digits) &&
        _allowed(special, spec.special);
  }

  bool _allowed(int actual, int expected) {
    if (actual > 0 && expected == 0) {
      return false;
    }

    return actual >= expected;
  }
}

class _DeterministicByteStream {
  _DeterministicByteStream._({
    required AesCtr aesCtr,
    required SecretKey secretKey,
  })  : _aesCtr = aesCtr,
        _secretKey = secretKey;

  final AesCtr _aesCtr;
  final SecretKey _secretKey;

  Uint8List _buffer = Uint8List(0);
  int _position = 0;

  static Future<_DeterministicByteStream> create({
    required String password,
    required String realm,
  }) async {
    final pbkdf2 = Pbkdf2(
      macAlgorithm: Hmac.sha256(),
      iterations: 4096,
      bits: 256,
    );

    final derivedKey = await pbkdf2.deriveKey(
      secretKey: SecretKey(utf8.encode(password)),
      nonce: utf8.encode(realm),
    );

    return _DeterministicByteStream._(
      aesCtr: AesCtr.with256bits(macAlgorithm: MacAlgorithm.empty),
      secretKey: derivedKey,
    );
  }

  Future<void> warmUp([int minimumBytes = 4096]) async {
    if (_buffer.length >= minimumBytes) {
      return;
    }

    _buffer = await _generateBytes(minimumBytes);
  }

  int readByte() {
    if (_position >= _buffer.length) {
      throw const MyKeyException('随机数据不足，无法生成密码');
    }

    final value = _buffer[_position];
    _position += 1;
    return value;
  }

  Future<Uint8List> _generateBytes(int length) async {
    final secretBox = await _aesCtr.encrypt(
      Uint8List(length),
      secretKey: _secretKey,
      nonce: Uint8List(16),
    );

    return Uint8List.fromList(secretBox.cipherText);
  }
}

bool _isUpperRune(int rune) => rune >= 0x41 && rune <= 0x5A;

bool _isLowerRune(int rune) => rune >= 0x61 && rune <= 0x7A;

bool _isDigitRune(int rune) => rune >= 0x30 && rune <= 0x39;

bool _isAllowedSpecialRune(int rune) => _isAsciiSymbolOrPunctuation(rune);

bool _isSpecialRune(int rune) => _isAsciiSymbolOrPunctuation(rune);

bool _isAsciiSymbolOrPunctuation(int rune) {
  if (rune < 0x21 || rune > 0x7E) {
    return false;
  }

  return !_isUpperRune(rune) && !_isLowerRune(rune) && !_isDigitRune(rune);
}
