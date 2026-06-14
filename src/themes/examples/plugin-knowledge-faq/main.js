// plugin-knowledge-faq entry point
class FaqKnowledge {
  constructor() {
    this.name = "faq";
  }

  async query(q) {
    const data = { entries: [] };
    try {
      const resp = await fetch("./data.json");
      const json = await resp.json();
      return json.entries.filter(e =>
        e.question.toLowerCase().includes(q.toLowerCase())
      );
    } catch {
      return [];
    }
  }
}

export default FaqKnowledge;