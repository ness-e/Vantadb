"""
VantaDB Retriever for DSPy

This example demonstrates how to use VantaDB as a retriever for DSPy.
VantaDB provides high-performance hybrid vector + text search optimized
for retrieval-augmented generation in DSPy pipelines.
"""

import vantadb_py as vantadb
from typing import List, Dict, Any, Optional, Union
import os

DB_PATH = "./dspy_vantadb_db"

class VantaDBRetriever:
    """
    VantaDB-based retriever for DSPy.
    
    Implements DSPy's retriever interface using VantaDB's hybrid search.
    """
    
    def __init__(
        self,
        db_path: str = DB_PATH,
        namespace: str = "dspy/documents",
        k: int = 5
    ):
        """
        Initialize VantaDB retriever.
        
        Args:
            db_path: Path to VantaDB database
            namespace: Namespace for documents
            k: Number of documents to retrieve
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=1_000_000_000)
        self.namespace = namespace
        self.k = k
        
    def add(
        self,
        documents: List[Dict[str, Any]],
        batch_size: int = 100
    ) -> int:
        """
        Add documents to the retriever.
        
        Args:
            documents: List of document dictionaries with 'text' and optional 'metadata'
            batch_size: Batch size for writes
            
        Returns:
            Number of documents added
        """
        count = 0
        for doc in documents:
            doc_id = doc.get("id") or f"doc-{count}"
            text = doc.get("text", "")
            metadata = doc.get("metadata", {})
            vector = doc.get("embedding")
            
            # Add document metadata
            metadata["_document_id"] = doc_id
            
            self.db.put(
                self.namespace,
                doc_id,
                text,
                metadata=metadata,
                vector=vector
            )
            count += 1
        
        return count
    
    def retrieve(
        self,
        query: str,
        query_vector: Optional[List[float]] = None,
        k: Optional[int] = None,
        filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Retrieve documents for the query.
        
        Args:
            query: Query string
            query_vector: Optional query embedding for semantic search
            k: Number of documents to retrieve (uses self.k if not provided)
            filters: Optional metadata filters
            
        Returns:
            List of retrieved documents with scores
        """
        top_k = k or self.k
        search_filters = filters or {}
        
        hits = self.db.search_memory(
            self.namespace,
            query_vector=query_vector,
            text_query=query,
            top_k=top_k,
            filters=search_filters
        )
        
        documents = []
        for hit in hits:
            documents.append({
                "text": hit.payload,
                "metadata": dict(hit.metadata),
                "score": hit.score
            })
        
        return documents
    
    def __call__(
        self,
        query: str,
        query_vector: Optional[List[float]] = None,
        k: Optional[int] = None
    ) -> List[Dict[str, Any]]:
        """
        Call the retriever (DSPy interface).
        
        Args:
            query: Query string
            query_vector: Optional query embedding
            k: Number of documents to retrieve
            
        Returns:
            List of retrieved documents
        """
        return self.retrieve(query, query_vector, k)
    
    def get_document(self, doc_id: str) -> Optional[Dict[str, Any]]:
        """
        Get a document by ID.
        
        Args:
            doc_id: Document identifier
            
        Returns:
            Document or None
        """
        record = self.db.get(self.namespace, doc_id)
        if record:
            return {
                "text": record["payload"],
                "metadata": record["metadata"]
            }
        return None
    
    def delete(self, doc_id: str) -> bool:
        """
        Delete a document.
        
        Args:
            doc_id: Document identifier
            
        Returns:
            True if deleted
        """
        return self.db.delete(self.namespace, doc_id)
    
    def get_document_count(self) -> int:
        """
        Get total document count.
        
        Returns:
            Number of documents
        """
        records = self.db.list(self.namespace, {"limit": 1000000})
        return len(records)
    
    def clear(self) -> int:
        """
        Clear all documents.
        
        Returns:
            Number of deleted documents
        """
        documents = self.db.list(self.namespace, {"limit": 1000000})
        count = 0
        for doc in documents:
            if self.db.delete(self.namespace, doc["key"]):
                count += 1
        return count
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB retriever for DSPy."""
    
    retriever = VantaDBRetriever(namespace="dspy/documents", k=3)
    
    print("📝 Adding documents...")
    
    # Sample documents
    documents = [
        {
            "id": "doc-001",
            "text": "VantaDB is an embedded persistent memory and vector retrieval engine for local-first AI applications.",
            "metadata": {"category": "database", "type": "description"}
        },
        {
            "id": "doc-002",
            "text": "DSPy is a framework for algorithmically optimizing LM prompts and weights.",
            "metadata": {"category": "framework", "type": "description"}
        },
        {
            "id": "doc-003",
            "text": "Retrieval-Augmented Generation (RAG) combines retrieval with generation for better AI responses.",
            "metadata": {"category": "ai", "type": "concept"}
        },
        {
            "id": "doc-004",
            "text": "Vector databases enable efficient similarity search for high-dimensional embeddings.",
            "metadata": {"category": "database", "type": "concept"}
        },
        {
            "id": "doc-005",
            "text": "DSPy provides a declarative interface for building LLM pipelines with automatic optimization.",
            "metadata": {"category": "framework", "type": "feature"}
        }
    ]
    
    count = retriever.add(documents)
    print(f"  Added {count} documents")
    
    print("\n🔍 Retrieving for 'vector database'...")
    results = retriever.retrieve("vector database")
    for i, doc in enumerate(results, 1):
        print(f"  [{i}] Score: {doc['score']:.3f}")
        print(f"      Text: {doc['text'][:80]}...")
    
    print("\n🔍 Calling retriever directly (DSPy interface) for 'DSPy'...")
    results = retriever("DSPy")
    for i, doc in enumerate(results, 1):
        print(f"  [{i}] Score: {doc['score']:.3f}")
        print(f"      Text: {doc['text'][:80]}...")
    
    print(f"\n📊 Total documents: {retriever.get_document_count()}")
    
    print("\n🧹 Cleaning up...")
    retriever.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
