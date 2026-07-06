"""
VantaDB Integration for Mem0

This example demonstrates how to use VantaDB as the persistence backend for Mem0.
VantaDB provides a robust, high-performance storage engine for Mem0's memory operations.
"""

import vantadb_py as vantadb
from typing import List, Dict, Any, Optional
import json
import os

DB_PATH = "./mem0_vantadb_db"

class VantaDBMem0Backend:
    """
    VantaDB backend for Mem0 memory operations.
    
    Implements Mem0's storage interface using VantaDB's hybrid vector + text search.
    """
    
    def __init__(self, db_path: str = DB_PATH, namespace: str = "mem0/memories"):
        """
        Initialize VantaDB backend for Mem0.
        
        Args:
            db_path: Path to VantaDB database
            namespace: Namespace for Mem0 memories
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=512_000_000)
        self.namespace = namespace
        
    def add(
        self,
        content: str,
        metadata: Optional[Dict[str, Any]] = None,
        user_id: Optional[str] = None,
        vector: Optional[List[float]] = None
    ) -> Dict[str, Any]:
        """
        Add a memory to VantaDB.
        
        Args:
            content: Memory content
            metadata: Optional metadata
            user_id: Optional user identifier
            vector: Optional embedding vector
            
        Returns:
            Created memory record
        """
        # Generate unique key
        import uuid
        key = f"{user_id or 'default'}-{uuid.uuid4().hex[:8]}"
        
        # Merge user_id into metadata
        meta = metadata or {}
        if user_id:
            meta["user_id"] = user_id
        
        record = self.db.put(
            self.namespace,
            key,
            content,
            metadata=meta,
            vector=vector
        )
        
        return {
            "id": key,
            "content": content,
            "metadata": meta,
            "created_at": record["created_at_ms"],
            "updated_at": record["updated_at_ms"]
        }
    
    def get(self, memory_id: str) -> Optional[Dict[str, Any]]:
        """
        Retrieve a memory by ID.
        
        Args:
            memory_id: Memory identifier
            
        Returns:
            Memory record or None
        """
        record = self.db.get(self.namespace, memory_id)
        if record:
            return {
                "id": record["key"],
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
        user_id: Optional[str] = None,
        limit: int = 10,
        filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Search memories with hybrid vector + text search.
        
        Args:
            query: Text query
            query_vector: Optional vector for semantic search
            user_id: Optional user filter
            limit: Maximum results
            filters: Additional metadata filters
            
        Returns:
            List of matching memories
        """
        # Build filters
        search_filters = filters or {}
        if user_id:
            search_filters["user_id"] = user_id
        
        hits = self.db.search_memory(
            self.namespace,
            query_vector=query_vector,
            text_query=query,
            top_k=limit,
            filters=search_filters
        )
        
        results = []
        for hit in hits:
            results.append({
                "id": hit.key,
                "content": hit.payload,
                "metadata": dict(hit.metadata),
                "score": hit.score,
                "created_at": hit.created_at_ms
            })
        
        return results
    
    def update(
        self,
        memory_id: str,
        content: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> Optional[Dict[str, Any]]:
        """
        Update a memory.
        
        Args:
            memory_id: Memory identifier
            content: New content (optional)
            metadata: New metadata (optional)
            
        Returns:
            Updated memory or None
        """
        existing = self.get(memory_id)
        if not existing:
            return None
        
        new_content = content or existing["content"]
        new_metadata = metadata or existing["metadata"]
        
        record = self.db.put(
            self.namespace,
            memory_id,
            new_content,
            metadata=new_metadata
        )
        
        return {
            "id": memory_id,
            "content": new_content,
            "metadata": new_metadata,
            "created_at": record["created_at_ms"],
            "updated_at": record["updated_at_ms"]
        }
    
    def delete(self, memory_id: str) -> bool:
        """
        Delete a memory.
        
        Args:
            memory_id: Memory identifier
            
        Returns:
            True if deleted
        """
        return self.db.delete(self.namespace, memory_id)
    
    def get_all(
        self,
        user_id: Optional[str] = None,
        limit: int = 100
    ) -> List[Dict[str, Any]]:
        """
        Get all memories for a user.
        
        Args:
            user_id: Optional user filter
            limit: Maximum results
            
        Returns:
            List of memories
        """
        filters = {"user_id": user_id} if user_id else {}
        records = self.db.list(self.namespace, {"limit": limit, "filters": filters})
        
        return [
            {
                "id": r["key"],
                "content": r["payload"],
                "metadata": r["metadata"],
                "created_at": r["created_at_ms"]
            }
            for r in records
        ]
    
    def delete_all(self, user_id: Optional[str] = None) -> int:
        """
        Delete all memories for a user.
        
        Args:
            user_id: Optional user filter
            
        Returns:
            Number of deleted memories
        """
        memories = self.get_all(user_id=user_id, limit=1000)
        count = 0
        for mem in memories:
            if self.delete(mem["id"]):
                count += 1
        return count
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB backend for Mem0."""
    
    backend = VantaDBMem0Backend(namespace="mem0/memories")
    
    print("📝 Adding memories...")
    
    # Add memories for different users
    backend.add(
        "User prefers dark mode in all applications",
        user_id="user-001",
        metadata={"category": "preference", "priority": "high"}
    )
    
    backend.add(
        "User is working on a machine learning project with PyTorch",
        user_id="user-001",
        metadata={"category": "project", "status": "active"}
    )
    
    backend.add(
        "User needs help with data visualization using matplotlib",
        user_id="user-002",
        metadata={"category": "task", "priority": "medium"}
    )
    
    print("\n🔍 Searching for 'dark mode'...")
    results = backend.search("dark mode", user_id="user-001")
    for result in results:
        print(f"  Score: {result['score']:.3f}")
        print(f"  Content: {result['content']}")
        print(f"  Metadata: {result['metadata']}")
    
    print("\n🔍 Getting all memories for user-001...")
    memories = backend.get_all(user_id="user-001")
    for mem in memories:
        print(f"  [{mem['id']}] {mem['content'][:60]}...")
    
    print("\n🧹 Cleaning up...")
    backend.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
