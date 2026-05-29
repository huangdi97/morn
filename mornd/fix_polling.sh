#!/bin/bash
cd /home/hermes/morn/mornd
opencode run "修复 telegram_bot.py 中 start() 方法的 aiogram polling 问题。当前 polling 启动后立即停止（日志显示 'Start polling' 后立即 'Polling stopped'），导致无法接收消息。需要改成手动 aiohttp getUpdates 长轮询，替代 aiogram 的 start_polling。具体：

1. 保留 BirthGuide 类不变
2. 修改 TelegramBot 类：去掉 self.dp.start_polling()，改为手动 polling 循环
3. 手动 polling 实现：用 aiohttp 的 ClientSession，每 2 秒 GET getUpdates?timeout=30&offset=X
4. 解析响应中的 updates，匹配 Command 和普通消息
5. 用 self.bot.send_message(chat_id, text) 回复
6. 维护 offset 变量（每次处理完 update 后 offset = update_id + 1）
7. 保持所有 handler 逻辑不变（cmd_start, cmd_status, cmd_forget, cmd_clear, cmd_help, handle_message）

start() 方法改为：
async def start(self):
    offset = 0
    while not self._stop_polling:
        try:
            async with self.http_session.get(
                f'https://api.telegram.org/bot{self.token}/getUpdates',
                params={'offset': offset, 'timeout': 30},
                timeout=35
            ) as resp:
                data = await resp.json()
                for update in data.get('result', []):
                    if 'message' in update:
                        await self._dispatch(update['message'])
                    offset = update['update_id'] + 1
        except asyncio.CancelledError:
            break
        except Exception as e:
            self._logger.error(f'poll error: {e}')
            await asyncio.sleep(2)

_dispatch() 方法根据消息内容分发给对应 handler。
停止时设置 self._stop_polling = True。
在 __init__ 中创建 self.http_session 和 self._stop_polling。

完成后运行 python3 -c 'from morn_core.presence.telegram_bot import BirthGuide, TelegramBot; print(\"import OK\")' 和 python3 -m pytest tests/test_telegram_bot.py -v。" -f /home/hermes/.hermes/cache/documents/morn_phase5_telegram_bot.txt --model deepseek/deepseek-v4-flash
