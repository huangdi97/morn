class WeChatChannel {
  constructor(config) {
    this.webhookUrl = config.webhookUrl;
  }

  async receiveMessage(raw) {
    const parsed = JSON.parse(raw);
    return {
      id: parsed.MsgId,
      from: parsed.FromUserName,
      content: parsed.Content,
      timestamp: parsed.CreateTime,
    };
  }

  async sendMessage(msg) {
    const payload = {
      msgtype: "markdown",
      markdown: { content: msg.content },
    };
    const resp = await fetch(this.webhookUrl, {
      method: "POST",
      body: JSON.stringify(payload),
    });
    return resp.ok;
  }
}

module.exports = { WeChatChannel };