import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'src/mykey_service.dart';

void main() {
  runApp(const MyKeyApp());
}

class MyKeyApp extends StatelessWidget {
  const MyKeyApp({super.key});

  @override
  Widget build(BuildContext context) {
    const seedColor = Color(0xFF14B8A6);

    return MaterialApp(
      title: 'MyKey',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: seedColor,
          brightness: Brightness.dark,
        ),
        scaffoldBackgroundColor: const Color(0xFF07131F),
        useMaterial3: true,
        inputDecorationTheme: const InputDecorationTheme(
          border: OutlineInputBorder(
            borderRadius: BorderRadius.all(Radius.circular(18)),
          ),
        ),
      ),
      home: const MyKeyPage(),
    );
  }
}

class MyKeyPage extends StatefulWidget {
  const MyKeyPage({super.key});

  @override
  State<MyKeyPage> createState() => _MyKeyPageState();
}

class _MyKeyPageState extends State<MyKeyPage> {
  final _masterController = TextEditingController();
  final _serviceController = TextEditingController();
  final _resultController = TextEditingController();
  final _service = const MyKeyService();

  bool _busy = false;
  bool _showMasterPassword = false;
  String _status = '准备就绪';

  @override
  void dispose() {
    _masterController.dispose();
    _serviceController.dispose();
    _resultController.dispose();
    super.dispose();
  }

  Future<void> _calculatePassword() async {
    setState(() {
      _busy = true;
      _status = '正在计算密码...';
      _resultController.text = '';
    });

    try {
      final password = await _service.derivePassword(
        masterPassword: _masterController.text,
        service: _serviceController.text,
      );

      if (!mounted) {
        return;
      }

      setState(() {
        _resultController.text = password;
        _status = '密码计算完成';
      });
    } on MyKeyException catch (error) {
      if (!mounted) {
        return;
      }

      setState(() {
        _status = error.message;
      });
    } catch (error) {
      if (!mounted) {
        return;
      }

      setState(() {
        _status = '计算失败：$error';
      });
    } finally {
      if (mounted) {
        setState(() {
          _busy = false;
        });
      }
    }
  }

  Future<void> _copyPassword() async {
    if (_resultController.text.isEmpty) {
      setState(() {
        _status = '请先计算密码';
      });
      return;
    }

    await Clipboard.setData(ClipboardData(text: _resultController.text));

    if (!mounted) {
      return;
    }

    setState(() {
      _status = '密码已复制到剪贴板';
    });
  }

  @override
  Widget build(BuildContext context) {
    final isError = _status.contains('失败') ||
        _status.contains('请先') ||
        _status.contains('不能为空');

    return Scaffold(
      body: Container(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [
              Color(0xFF07131F),
              Color(0xFF0D1B2F),
              Color(0xFF11253D),
            ],
          ),
        ),
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 460),
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: const Color(0xCC091323),
                  borderRadius: BorderRadius.circular(28),
                  border: Border.all(color: Colors.white12),
                  boxShadow: const [
                    BoxShadow(
                      color: Color(0x73080F1D),
                      blurRadius: 50,
                      offset: Offset(0, 20),
                    ),
                  ],
                ),
                child: Padding(
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        'MyKey',
                        style:
                            Theme.of(context).textTheme.displaySmall?.copyWith(
                                  fontWeight: FontWeight.w700,
                                  color: Colors.white,
                                ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Flutter + 纯 Dart 重写 Cloudflare gokey 密码派生逻辑',
                        style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                              color: const Color(0xFFBED7FF),
                            ),
                      ),
                      const SizedBox(height: 22),
                      TextField(
                        controller: _masterController,
                        obscureText: !_showMasterPassword,
                        enabled: !_busy,
                        decoration: InputDecoration(
                          labelText: '主密码',
                          hintText: '请输入主密码',
                          suffixIcon: IconButton(
                            tooltip: _showMasterPassword ? '隐藏密码' : '显示密码',
                            onPressed: _busy
                                ? null
                                : () {
                                    setState(() {
                                      _showMasterPassword =
                                          !_showMasterPassword;
                                    });
                                  },
                            icon: Icon(
                              _showMasterPassword
                                  ? Icons.visibility_off_outlined
                                  : Icons.visibility_outlined,
                            ),
                          ),
                        ),
                      ),
                      const SizedBox(height: 16),
                      TextField(
                        controller: _serviceController,
                        enabled: !_busy,
                        decoration: const InputDecoration(
                          labelText: '服务标识',
                          hintText: '例如 github.com',
                        ),
                      ),
                      const SizedBox(height: 16),
                      TextField(
                        controller: _resultController,
                        readOnly: true,
                        decoration: const InputDecoration(
                          labelText: '计算结果',
                          hintText: '点击计算按钮后显示',
                        ),
                      ),
                      const SizedBox(height: 12),
                      Text(
                        '固定规则：16 位，至少包含 1 个大写、1 个小写、1 个数字和 1 个特殊字符。',
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: const Color(0xFF93C5FD),
                            ),
                      ),
                      const SizedBox(height: 20),
                      Row(
                        children: [
                          Expanded(
                            child: FilledButton(
                              onPressed: _busy ? null : _calculatePassword,
                              child: Text(_busy ? '计算中...' : '计算密码'),
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: OutlinedButton(
                              onPressed: _busy ? null : _copyPassword,
                              child: const Text('复制密码'),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 18),
                      Text(
                        _status,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: isError
                                  ? const Color(0xFFFCA5A5)
                                  : const Color(0xFFD1FAE5),
                            ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
