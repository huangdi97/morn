#!/bin/bash
cd /home/hermes/morn/mornd
cp /home/hermes/.hermes/cache/documents/morn_phase6_user_protection.txt phase6_instr.txt
opencode run "根据 phase6_instr.txt 在 morn_core/security/user_protection.py 实现 UserProtection 类，在 tests/test_user_protection.py 实现测试。完成后运行 pytest tests/test_user_protection.py -v。" -f phase6_instr.txt --model deepseek/deepseek-v4-flash
