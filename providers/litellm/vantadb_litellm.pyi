class VantaDBLiteLLM:
    def __init__(
        self,
        db_path: str,
        api_key: str | None = None,
        model: str = "text-embedding-3-small",
        namespace: str = "litellm_store",
        timeout: float | None = None,
    ) -> None: ...
    def embed(self, texts: list[str]) -> list[list[float]]: ...
    def search(
        self,
        namespace: str,
        query_embedding: list[float],
        text_query: str | None = None,
        filters: dict[str, str] | None = None,
        distance_metric: str | None = None,
        top_k: int = 10,
    ) -> list[dict]: ...
    def store(
        self,
        text: str,
        embedding: list[float],
        metadata: dict | None = None,
    ) -> str: ...
    def delete(self, key: str, namespace: str | None = None) -> bool: ...
    def get(self, namespace: str, key: str) -> dict | None: ...
    def list(self, namespace: str, limit: int = 100, cursor: int | None = None) -> dict: ...
    def list_namespaces(self) -> list[str]: ...

__version__: str
