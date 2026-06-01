import logging
from pathlib import Path

logger = logging.getLogger("morn.memory")


class VectorStore:
    def __init__(self, data_dir: Path):
        self._available = False
        self._collection = None
        try:
            import chromadb
            self._client = chromadb.PersistentClient(path=str(data_dir / "chroma_db"))
            self._collection = self._client.get_or_create_collection(name="morn_capsules")
            self._available = True
        except ImportError:
            logger.warning("chromadb not installed, vector search disabled")
        except Exception as e:
            logger.warning("chromadb init failed: %s", e)

    async def add_embedding(self, event_id: str, text: str):
        if not self._available:
            return
        try:
            self._collection.upsert(ids=[event_id], documents=[text])
        except Exception as e:
            logger.warning("add_embedding failed for %s: %s", event_id, e)

    async def search_similar(self, query: str, limit: int = 5) -> list[str]:
        if not self._available:
            return []
        try:
            results = self._collection.query(query_texts=[query], n_results=limit)
            ids = results.get("ids", [[]])[0]
            return list(ids) if ids else []
        except Exception:
            return []
