#!/bin/bash
cd /home/hermes/morn/mornd
opencode run "根据 phase5_instr.txt 在 morn_core/presence/telegram_bot.py 实现 BirthGuide 和 TelegramBot，在 tests/test_telegram_bot.py 实现测试。完成后运行 pytest tests/test_telegram_bot.py -v。" -f phase5_instr.txt --model deepseek/deepseek-v4-flash
