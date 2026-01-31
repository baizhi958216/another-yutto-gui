#!/usr/bin/env python3
"""
Yutto Wrapper - 修复 Windows 编码问题

这个 wrapper 在导入 yutto 之前强制设置 UTF-8 编码，
解决 Windows 上的 GBK 编码错误。
"""

import sys
import io
import os

# 在导入任何其他模块之前，强制设置 UTF-8 编码
if sys.platform == 'win32':
    # 重定向 stdout 和 stderr 到 UTF-8 编码的 TextIOWrapper
    if hasattr(sys.stdout, 'buffer'):
        sys.stdout = io.TextIOWrapper(
            sys.stdout.buffer,
            encoding='utf-8',
            errors='replace',
            line_buffering=True
        )
    if hasattr(sys.stderr, 'buffer'):
        sys.stderr = io.TextIOWrapper(
            sys.stderr.buffer,
            encoding='utf-8',
            errors='replace',
            line_buffering=True
        )

    # 设置环境变量
    os.environ['PYTHONIOENCODING'] = 'utf-8'
    os.environ['PYTHONUTF8'] = '1'

    # 禁用 colorama 的自动初始化
    os.environ['COLORAMA_AUTORESET'] = '0'
    os.environ['COLORAMA_STRIP'] = '1'

# 现在导入 yutto
from yutto.__main__ import main

if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        # 确保错误信息也使用 UTF-8 编码
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
