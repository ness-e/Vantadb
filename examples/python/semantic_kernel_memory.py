"""
VantaDB Memory Connector for Semantic Kernel

This example demonstrates how to use VantaDB as a memory connector for Microsoft Semantic Kernel.
VantaDB provides persistent, namespace-scoped memory with hybrid vector and text search
for AI applications built with Semantic Kernel.
"""

import vantadb_py as vantadb
from typing import List, Dict, Any, Optional
import json
import os

DB_PATH = "./semantic_kernel_vantadb_db"

class VantaDBSemanticMemory:
    """
    VantaDB-based memory connector for Semantic Kernel.
    
    Provides persistent memory storage with:
    - Collection-scoped isolation
    - Hybrid vector + text search
    - Metadata filtering
    - Semantic search capabilities
    """
    
    def __init__(self, db_path: str = DB_PATH, collection_name: str = "default"):
        """
        Initialize VantaDB memory connector.
        
        Args:
            db_path: Path to VantaDB database
            collection_name: Collection name for memory isolation
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=512_000_000)
        self.collection_name = collection_name
        self.namespace = f"semantic-kernel/{collection_name}"
        
    def save_information(
        self,
        text: str,
        key: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None,
        embedding: Optional[List[float]] = None
    ) -> str:
        """
        Save information to memory.
        
        Args:
            text: Text content to save
            key: Optional unique key
            metadata: Optional metadata
            embedding: Optional embedding vector
            
        Returns:
            Memory key
        """
        import uuid
        memory_key = key or f"mem-{uuid.uuid4().hex[:8]}"
        
        meta = metadata or {}
        meta.update({
            "collection": self.collection_name,
            "type": "information"
        })
        
        record = self.db.put(
            self.namespace,
            memory_key,
            text,
            metadata=meta,
            vector=embedding
        )
        
        return memory_key
    
    def save_reference(
        self,
        text: str,
        external_id: str,
        external_source: str,
        metadata: Optional[Dict[str, Any]] = None,
        embedding: Optional[List[float]] = None
    ) -> str:
        """
        Save a reference to external content.
        
        Args:
            text: Text content
            external_id: External identifier
            external_source: Source system
            metadata: Optional metadata
            embedding: Optional embedding vector
            
        Returns:
            Memory key
        """
        memory_key = f"ref-{external_source}-{external_id}"
        
        meta = metadata or {}
        meta.update({
            "collection": self.collection_name,
            "type": "reference",
            "external_id": external_id,
            "external_source": external_source
        })
        
        record = self.db.put(
            self.namespace,
            memory_key,
            text,
            metadata=meta,
            vector=embedding
        )
        
        return memory_key
    
    def retrieve(
        self,
        query: str,
        query_embedding: Optional[List[float]] = None,
        limit: int = 10,
        min_relevance_score: float = 0.0,
        filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Retrieve memories with hybrid search.
        
        Args:
            query: Search query
            query_embedding: Optional embedding for semantic search
            limit: Maximum results
            min_relevance_score: Minimum relevance threshold
            filters: Optional metadata filters
            
        Returns:
            List of matching memories
        """
        search_filters = filters or {}
        
        hits = self.db.search_memory(
            self.namespace,
            query_vector=query_embedding,
            text_query=query,
            top_k=limit,
            filters=search_filters
        )
        
        memories = []
        for hit in hits:
            if hit.score >= min_relevance_score:
                memories.append({
                    "key": hit.key,
                    "text": hit.payload,
                    "metadata": dict(hit.metadata),
                    "relevance": hit.score
                })
        
        return memories
    
    def get(self, key: str) -> Optional[Dict[str, Any]]:
        """
        Retrieve a memory by key.
        
        Args:
            key: Memory key
            
        Returns:
            Memory or None
        """
        record = self.db.get(self.namespace, key)
        if record:
            return {
                "key": record["key"],
                "text": record["payload"],
                "metadata": record["metadata"],
                "created_at": record["created_at_ms"]
            }
        return None
    
    def remove(self, key: str) -> bool:
        """
        Remove a memory.
        
        Args:
            key: Memory key
            
        Returns:
            True if removed
        """
        return self.db.delete(self.namespace, key)
    
    def list(
        self,
        limit: int = 100,
        filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        List memories in the collection.
        
        Args:
            limit: Maximum results
            filters: Optional metadata filters
            
        Returns:
            List of memories
        """
        records = self.db.list(self.namespace, {"limit": limit, "filters": filters or {}})
        
        return [
            {
                "key": r["key"],
                "text": r["payload"],
                "metadata": r["metadata"],
                "created_at": r["created_at_ms"]
            }
            for r in records
        ]
    
    def search_by_type(
        self,
        memory_type: str,
        query: str,
        limit: int = 10
    ) -> List[Dict[str, Any]]:
        """
        Search memories by type.
        
        Args:
            memory_type: Memory type (information, reference)
            query: Search query
            limit: Maximum results
            
        Returns:
            List of matching memories
        """
        return self.retrieve(
            query,
            limit=limit,
            filters={"type": memory_type}
        )
    
    def get_stats(self) -> Dict[str, Any]:
        """
        Get collection statistics.
        
        Returns:
            Statistics dictionary
        """
        memories = self.list(limit=100000)
        
        type_counts = {}
        for mem in memories:
            mem_type = mem["metadata"].get("type", "unknown")
            type_counts[mem_type] = type_counts.get(mem_type, 0) + 1
        
        return {
            "collection": self.collection_name,
            "total_memories": len(memories),
            "type_distribution": type_counts
        }
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB memory connector for Semantic Kernel."""
    
    memory = VantaDBSemanticMemory(collection_name="demo-app")
    
    print("📝 Saving information...")
    
    # Save information
    memory.save_information(
        "User prefers concise technical answers with code examples",
        metadata={"category": "preference", "priority": "high"}
    )
    
    memory.save_information(
        "User is working on a Python project using Semantic Kernel",
        metadata={"category": "project", "status": "active"}
    )
    
    # Save references
    memory.save_reference(
        "Semantic Kernel documentation: https://learn.microsoft.com/en-us/semantic-kernel/",
        external_id="sk-docs",
        external_source="microsoft",
        metadata={"type": "documentation"}
    )
    
    print("\n🔍 Retrieving memories for 'Semantic Kernel'...")
    results = memory.retrieve("Semantic Kernel", limit=5)
    for result in results:
        print(f"  Relevance: {result['relevance']:.3f}")
        print(f"  Text: {result['text'][:80]}...")
        print(f"  Metadata: {result['metadata']}")
    
    print("\n🔍 Searching by type (reference)...")
    references = memory.search_by_type("reference", "documentation")
    for ref in references:
        print(f"  {ref['text'][:80]}...")
    
    print("\n📊 Collection statistics:")
    stats = memory.get_stats()
    print(f"  Collection: {stats['collection']}")
    print(f"  Total memories: {stats['total_memories']}")
    print(f"  Type distribution: {stats['type_distribution']}")
    
    print("\n🧹 Cleaning up...")
    memory.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
