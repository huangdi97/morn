#!/bin/bash
cd /home/hermes/morn/mornd
cp /home/hermes/.hermes/cache/documents/morn_phase7_integration.txt phase7_instr.txt
opencode run "根据 phase7_instr.txt 修改 server.py（添加 load_config、修改 main、修改 cli_loop 串联所有模块），添加 examples/instance_config.json，创建 tests/test_integration.py。完成后运行 pytest tests/test_integration.py -v 和集成验证。" -f phase7_instr.txt --model deepseek/deepseek-v4-flash
