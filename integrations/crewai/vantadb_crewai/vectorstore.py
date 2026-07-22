from __future__ import annotations

from typing import Any, Callable, List, Optional

import vantadb_py as vanta

try:
    from crewai.tools import BaseTool as CrewAIBaseTool
except ImportError:
    CrewAIBaseTool = object  # fallback si crewai no está instalado

DEFAULT_NAMESPACE = "crewai"
DEFAULT_TOP_K = 4


class VantaDBTool(CrewAIBaseTool):
    def __init__(
        self,
        embedding: Optional[Callable[[str], List[float]]] = None,
        name: str = "VantaDB Search",
        description: str = "Search documents stored in VantaDB",
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        """Initialize a VantaDB-powered CrewAI tool.

        Args:
            embedding: Optional callable that converts text to a vector
                embedding list. Used for semantic search.
            name: Name for the CrewAI tool. Defaults to "VantaDB Search".
            description: Description for the CrewAI tool. Defaults to
                "Search documents stored in VantaDB".
            db_path: Filesystem path for the VantaDB database.
                Defaults to "./vantadb_data".
            namespace: VantaDB namespace to operate on.
                Defaults to "crewai".
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        super().__init__(name=name, description=description)
        # CrewAI BaseTool is a Pydantic v2 model with extra='forbid';
        # use object.__setattr__ to bypass Pydantic field validation.
        object.__setattr__(self, "namespace", namespace)
        object.__setattr__(self, "embedding", embedding)
        object.__setattr__(self, "_db", vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        ))

    def _run(self, query: str, **kwargs: Any) -> str:
        """Execute a search query against the VantaDB store.

        Implements the CrewAI ``BaseTool._run`` protocol. When an
        embedding function is configured, performs vector similarity search;
        otherwise falls back to listing all records.

        Args:
            query: The search query string.
            **kwargs: Additional keyword arguments. Supports ``k`` (int)
                to override the default top-K result count.

        Returns:
            A newline-separated string of result passages, or
            ``"No results found."`` if the store is empty.
        """
        if not query or not query.strip():
            return "No query provided."

        k = (
            kwargs.get("k", self.top_k)
            if hasattr(self, "top_k")
            else kwargs.get("k", DEFAULT_TOP_K)
        )

        if self.embedding:
            embedding = self.embedding(query)
            results = self._db.search_memory(
                self.namespace,
                embedding,
                top_k=k,
                distance_metric="cosine",
            )
            passages = (
                [hit.payload for hit in results]
                if hasattr(results, "__iter__")
                else []
            )
        else:
            # Fallback: list all
            results = self._db.list_memory(namespace=self.namespace, limit=k)
            records = (
                results.records
                if hasattr(results, "records")
                else list(results)
            )
            passages = [
                getattr(r, "payload", None) or str(r)
                for r in records
            ]

        return "\n".join(passages) if passages else "No results found."

    def _put(self, text: str, metadata: Optional[dict] = None) -> None:
        """Store a text entry with optional metadata in VantaDB.

        Implements the CrewAI ``BaseTool`` storage protocol. Generates a
        UUID key automatically. If an embedding function is configured, the
        vector is computed and stored alongside the text.

        Args:
            text: The text content to store. Must be non-empty.
            metadata: Optional dictionary of metadata to attach.

        Raises:
            ValueError: If ``text`` is empty or whitespace-only.
        """
        if not text or not text.strip():
            raise ValueError("Text cannot be empty")

        import uuid

        vector = None
        if self.embedding:
            vector = self.embedding(text)
        self._db.put(
            self.namespace,
            str(uuid.uuid4()),
            text,
            metadata=metadata or {},
            vector=vector,
        )

    def categorize(self, text: str) -> str:
        """Classify text into a predefined category based on keywords.

        Uses simple keyword matching to categorise the input as one of:
        ``"question"``, ``"technical"``, ``"greeting"``, or
        ``"informational"``.

        Args:
            text: The text to categorise.

        Returns:
            One of ``"empty"`` (if text is blank), ``"question"``,
            ``"technical"``, ``"greeting"``, or ``"informational"``.
        """
        if not text or not text.strip():
            return "empty"

        # Keyword-based categorization
        text_lower = text.lower()

        question_words = {
            "what",
            "how",
            "why",
            "when",
            "where",
            "who",
            "which",
            "can",
            "could",
            "would",
            "should",
        }
        if (
            any(text_lower.startswith(w) for w in question_words)
            or text_lower.endswith("?")
        ):
            return "question"

        technical_indicators = {
            "code",
            "error",
            "bug",
            "function",
            "api",
            "syntax",
            "compile",
            "debug",
            "exception",
        }
        if any(w in text_lower for w in technical_indicators):
            return "technical"

        greeting_indicators = {
            "hello",
            "hi",
            "hey",
            "greetings",
            "good morning",
            "good afternoon",
        }
        if any(text_lower.startswith(w) for w in greeting_indicators):
            return "greeting"

        return "informational"

    def __call__(self, *args: Any, **kwargs: Any) -> str:
        return self._run(*args, **kwargs)
