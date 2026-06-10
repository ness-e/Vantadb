"""
VantaDB Memory Integration for CrewAI

This example demonstrates how to use VantaDB as a memory backend for CrewAI agents.
VantaDB provides persistent, namespace-scoped memory with hybrid vector and text search.
"""

import vantadb_py as vantadb
from typing import List, Dict, Any, Optional
import os

DB_PATH = "./crewai_memory_db"

class VantaDBMemory:
    """
    VantaDB-based memory implementation for CrewAI agents.
    
    Provides persistent memory storage with:
    - Namespace-scoped isolation
    - Hybrid vector + text search
    - Metadata filtering
    - Automatic persistence
    """
    
    def __init__(self, db_path: str = DB_PATH, namespace: str = "crewai/agent"):
        """
        Initialize VantaDB memory backend.
        
        Args:
            db_path: Path to VantaDB database
            namespace: Namespace for this agent's memories
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=256_000_000)
        self.namespace = namespace
        
    def add(
        self,
        key: str,
        content: str,
        vector: Optional[List[float]] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Store a memory in VantaDB.
        
        Args:
            key: Unique identifier for the memory
            content: Text content of the memory
            vector: Optional embedding vector
            metadata: Optional metadata dictionary
            
        Returns:
            The stored memory record
        """
        meta = metadata or {}
        record = self.db.put(
            self.namespace,
            key,
            content,
            metadata=meta,
            vector=vector
        )
        return {
            "key": key,
            "content": content,
            "metadata": meta,
            "created_at": record["created_at_ms"]
        }
    
    def get(self, key: str) -> Optional[Dict[str, Any]]:
        """
        Retrieve a memory by key.
        
        Args:
            key: Memory key to retrieve
            
        Returns:
            Memory record or None if not found
        """
        record = self.db.get(self.namespace, key)
        if record:
            return {
                "key": record["key"],
                "content": record["payload"],
                "metadata": record["metadata"],
                "created_at": record["created_at_ms"],
                "updated_at": record["updated_at_ms"]
            }
        return None
    
    def search(
        self,
        query: str,
        query_vector: Optional[List[float]] = None,
        top_k: int = 5,
        filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Search memories using hybrid vector + text search.
        
        Args:
            query: Text query for BM25 search
            query_vector: Optional vector for semantic search
            top_k: Number of results to return
            filters: Optional metadata filters
            
        Returns:
            List of matching memory records with scores
        """
        hits = self.db.search_memory(
            self.namespace,
            query_vector=query_vector,
            text_query=query,
            top_k=top_k,
            filters=filters or {}
        )
        
        results = []
        for hit in hits:
            record = hit["record"]
            results.append({
                "key": record["key"],
                "content": record["payload"],
                "metadata": record["metadata"],
                "score": hit["score"],
                "created_at": record["created_at_ms"]
            })
        
        return results
    
    def delete(self, key: str) -> bool:
        """
        Delete a memory by key.
        
        Args:
            key: Memory key to delete
            
        Returns:
            True if deleted, False otherwise
        """
        return self.db.delete(self.namespace, key)
    
    def list(self, limit: int = 100, filters: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """
        List all memories in the namespace.
        
        Args:
            limit: Maximum number of records to return
            filters: Optional metadata filters
            
        Returns:
            List of memory records
        """
        options = {
            "limit": limit,
            "filters": filters or {}
        }
        records = self.db.list(self.namespace, options)
        
        return [
            {
                "key": r["key"],
                "content": r["payload"],
                "metadata": r["metadata"],
                "created_at": r["created_at_ms"]
            }
            for r in records
        ]
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB memory usage with CrewAI-like patterns."""
    
    # Initialize VantaDB memory backend
    memory = VantaDBMemory(namespace="crewai/agent-001")
    
    print("📝 Storing agent memories...")
    
    # Store task context
    memory.add(
        "task-001",
        "User requested a Python script for data processing with pandas",
        metadata={"type": "task", "priority": "high", "status": "pending"}
    )
    
    # Store preferences
    memory.add(
        "pref-001",
        "User prefers verbose output with detailed error messages",
        metadata={"type": "preference", "category": "output"}
    )
    
    # Store context
    memory.add(
        "ctx-001",
        "Project uses Python 3.11 and requires pandas 2.0+",
        metadata={"type": "context", "category": "environment"}
    )
    
    print("\n🔍 Searching for 'pandas'...")
    results = memory.search("pandas", top_k=3)
    for i, result in enumerate(results, 1):
        print(f"  [{i}] Score: {result['score']:.3f}")
        print(f"      Content: {result['content']}")
        print(f"      Metadata: {result['metadata']}")
    
    print("\n🔍 Searching with metadata filter (type=preference)...")
    results = memory.search("", filters={"type": "preference"})
    for result in results:
        print(f"  Content: {result['content']}")
        print(f"  Metadata: {result['metadata']}")
    
    print("\n📋 Listing all memories...")
    all_memories = memory.list(limit=10)
    for mem in all_memories:
        print(f"  [{mem['key']}] {mem['content'][:50]}...")
    
    print("\n🧹 Cleaning up...")
    memory.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
