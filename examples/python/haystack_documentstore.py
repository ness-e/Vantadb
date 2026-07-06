"""
VantaDB DocumentStore for Haystack

This example demonstrates how to use VantaDB as a DocumentStore for Haystack.
VantaDB provides a high-performance, hybrid vector + text search backend optimized
for document retrieval in RAG pipelines.
"""

import vantadb_py as vantadb
from typing import List, Dict, Any, Optional, Union
import json
import os

DB_PATH = "./haystack_vantadb_db"

class VantaDBDocumentStore:
    """
    VantaDB-based DocumentStore for Haystack.
    
    Implements Haystack's DocumentStore interface using VantaDB's hybrid search.
    """
    
    def __init__(self, db_path: str = DB_PATH, namespace: str = "haystack/documents"):
        """
        Initialize VantaDB DocumentStore.
        
        Args:
            db_path: Path to VantaDB database
            namespace: Namespace for documents
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=1_000_000_000)
        self.namespace = namespace
        
    def write_documents(
        self,
        documents: List[Dict[str, Any]],
        batch_size: int = 100
    ) -> int:
        """
        Write documents to VantaDB.
        
        Args:
            documents: List of document dictionaries
            batch_size: Batch size for writes
            
        Returns:
            Number of documents written
        """
        count = 0
        for doc in documents:
            # Extract required fields
            doc_id = doc.get("id") or f"doc-{count}"
            content = doc.get("content", "")
            meta = doc.get("meta", {})
            vector = doc.get("embedding")
            
            # Add document metadata
            meta["_document_id"] = doc_id
            
            # Store in VantaDB
            self.db.put(
                self.namespace,
                doc_id,
                content,
                metadata=meta,
                vector=vector
            )
            count += 1
        
        return count
    
    def get_documents_by_id(
        self,
        ids: List[str],
        return_embedding: bool = False
    ) -> List[Dict[str, Any]]:
        """
        Retrieve documents by ID.
        
        Args:
            ids: List of document IDs
            return_embedding: Whether to return embeddings
            
        Returns:
            List of documents
        """
        documents = []
        for doc_id in ids:
            record = self.db.get(self.namespace, doc_id)
            if record:
                doc = {
                    "id": record["key"],
                    "content": record["payload"],
                    "meta": record["metadata"]
                }
                if return_embedding:
                    doc["embedding"] = record.get("vector")
                documents.append(doc)
        return documents
    
    def get_document_by_id(
        self,
        id: str,
        return_embedding: bool = False
    ) -> Optional[Dict[str, Any]]:
        """
        Retrieve a single document by ID.
        
        Args:
            id: Document ID
            return_embedding: Whether to return embedding
            
        Returns:
            Document or None
        """
        docs = self.get_documents_by_id([id], return_embedding)
        return docs[0] if docs else None
    
    def query(
        self,
        query: str,
        query_vector: Optional[List[float]] = None,
        filters: Optional[Dict[str, Any]] = None,
        top_k: int = 10,
        return_embedding: bool = False
    ) -> List[Dict[str, Any]]:
        """
        Query documents with hybrid search.
        
        Args:
            query: Text query
            query_vector: Optional vector for semantic search
            filters: Optional metadata filters
            top_k: Maximum results
            return_embedding: Whether to return embeddings
            
        Returns:
            List of matching documents with scores
        """
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
            doc = {
                "id": hit.key,
                "content": hit.payload,
                "meta": dict(hit.metadata),
                "score": hit.score,
            }
            if return_embedding:
                doc["embedding"] = list(hit.vector) if hit.vector else None
            documents.append(doc)
        
        return documents
    
    def delete_documents(
        self,
        ids: Optional[List[str]] = None,
        filters: Optional[Dict[str, Any]] = None
    ) -> int:
        """
        Delete documents.
        
        Args:
            ids: Optional list of document IDs
            filters: Optional metadata filters
            
        Returns:
            Number of deleted documents
        """
        if ids:
            count = 0
            for doc_id in ids:
                if self.db.delete(self.namespace, doc_id):
                    count += 1
            return count
        
        if filters:
            # Get all documents matching filters
            records = self.db.list(self.namespace, {"limit": 1000, "filters": filters})
            count = 0
            for record in records:
                if self.db.delete(self.namespace, record["key"]):
                    count += 1
            return count
        
        return 0
    
    def get_document_count(self) -> int:
        """
        Get total document count.
        
        Returns:
            Number of documents
        """
        records = self.db.list(self.namespace, {"limit": 1000000})
        return len(records)
    
    def get_all_documents(
        self,
        filters: Optional[Dict[str, Any]] = None,
        return_embedding: bool = False,
        batch_size: int = 100
    ) -> List[Dict[str, Any]]:
        """
        Get all documents.
        
        Args:
            filters: Optional metadata filters
            return_embedding: Whether to return embeddings
            batch_size: Batch size for retrieval
            
        Returns:
            List of all documents
        """
        records = self.db.list(self.namespace, {"limit": 1000000, "filters": filters or {}})
        
        documents = []
        for record in records:
            doc = {
                "id": record["key"],
                "content": record["payload"],
                "meta": record["metadata"]
            }
            if return_embedding:
                doc["embedding"] = record.get("vector")
            documents.append(doc)
        
        return documents
    
    def update_document(
        self,
        id: str,
        content: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        embedding: Optional[List[float]] = None
    ) -> bool:
        """
        Update a document.
        
        Args:
            id: Document ID
            content: New content (optional)
            meta: New metadata (optional)
            embedding: New embedding (optional)
            
        Returns:
            True if updated
        """
        existing = self.db.get(self.namespace, id)
        if not existing:
            return False
        
        new_content = content or existing["payload"]
        new_meta = meta or existing["metadata"]
        new_vector = embedding or existing.get("vector")
        
        self.db.put(
            self.namespace,
            id,
            new_content,
            metadata=new_meta,
            vector=new_vector
        )
        
        return True
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB DocumentStore for Haystack."""
    
    store = VantaDBDocumentStore(namespace="haystack/documents")
    
    print("📝 Writing documents...")
    
    # Sample documents
    documents = [
        {
            "id": "doc-001",
            "content": "VantaDB is an embedded persistent memory and vector retrieval engine for local-first AI applications.",
            "meta": {"category": "database", "type": "description"}
        },
        {
            "id": "doc-002",
            "content": "Haystack is an open-source NLP framework that enables developers to build powerful NLP applications.",
            "meta": {"category": "framework", "type": "description"}
        },
        {
            "id": "doc-003",
            "content": "Vector databases enable efficient similarity search for high-dimensional embeddings.",
            "meta": {"category": "database", "type": "concept"}
        },
        {
            "id": "doc-004",
            "content": "RAG (Retrieval-Augmented Generation) combines retrieval with generation for better AI responses.",
            "meta": {"category": "ai", "type": "concept"}
        }
    ]
    
    count = store.write_documents(documents)
    print(f"  Written {count} documents")
    
    print("\n🔍 Querying for 'vector database'...")
    results = store.query("vector database", top_k=3)
    for i, doc in enumerate(results, 1):
        print(f"  [{i}] Score: {doc['score']:.3f}")
        print(f"      ID: {doc['id']}")
        print(f"      Content: {doc['content'][:80]}...")
        print(f"      Meta: {doc['meta']}")
    
    print("\n🔍 Querying with metadata filter (category=framework)...")
    results = store.query("", filters={"category": "framework"})
    for doc in results:
        print(f"  ID: {doc['id']}")
        print(f"  Content: {doc['content'][:80]}...")
    
    print(f"\n📊 Total documents: {store.get_document_count()}")
    
    print("\n🧹 Cleaning up...")
    store.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
