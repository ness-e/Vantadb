"""
VantaDB Checkpoint Store for LangGraph

This example demonstrates how to use VantaDB as a CheckpointStore for LangGraph.
VantaDB provides persistent checkpoint storage with namespace isolation and
efficient retrieval for state management in LangGraph workflows.
"""

import vantadb_py as vantadb
from typing import Dict, Any, Optional, List
import json
import os

DB_PATH = "./langgraph_vantadb_db"

class VantaDBCheckpointStore:
    """
    VantaDB-based CheckpointStore for LangGraph.
    
    Provides persistent checkpoint storage with:
    - Thread-scoped isolation
    - Efficient checkpoint retrieval
    - Metadata for checkpoint versions
    - Namespace separation for different workflows
    """
    
    def __init__(self, db_path: str = DB_PATH, thread_id: str = "default"):
        """
        Initialize VantaDB CheckpointStore.
        
        Args:
            db_path: Path to VantaDB database
            thread_id: Thread identifier for this workflow
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=512_000_000)
        self.thread_id = thread_id
        self.namespace = f"langgraph/checkpoints-{thread_id}"
        
    def put(
        self,
        config: Dict[str, Any],
        checkpoint: Dict[str, Any],
        metadata: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Store a checkpoint.
        
        Args:
            config: Configuration dictionary
            checkpoint: Checkpoint state
            metadata: Optional metadata
            
        Returns:
            Stored checkpoint record
        """
        # Generate checkpoint ID from config
        checkpoint_id = config.get("checkpoint_id") or f"ckpt-{config.get('thread_id', 'default')}"
        
        # Merge metadata
        meta = metadata or {}
        meta.update({
            "thread_id": self.thread_id,
            "checkpoint_id": checkpoint_id,
            "config": json.dumps(config)
        })
        
        # Store checkpoint
        record = self.db.put(
            self.namespace,
            checkpoint_id,
            json.dumps(checkpoint),
            metadata=meta
        )
        
        return {
            "id": checkpoint_id,
            "config": config,
            "checkpoint": checkpoint,
            "metadata": meta,
            "created_at": record["created_at_ms"]
        }
    
    def get(self, config: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Retrieve a checkpoint by config.
        
        Args:
            config: Configuration dictionary
            
        Returns:
            Checkpoint or None
        """
        checkpoint_id = config.get("checkpoint_id")
        if not checkpoint_id:
            return None
        
        record = self.db.get(self.namespace, checkpoint_id)
        if record:
            try:
                checkpoint = json.loads(record["payload"])
                return {
                    "id": record["key"],
                    "config": json.loads(record["metadata"].get("config", "{}")),
                    "checkpoint": checkpoint,
                    "metadata": record["metadata"],
                    "created_at": record["created_at_ms"]
                }
            except json.JSONDecodeError:
                return None
        return None
    
    def list(
        self,
        config: Optional[Dict[str, Any]] = None,
        limit: int = 100
    ) -> List[Dict[str, Any]]:
        """
        List checkpoints.
        
        Args:
            config: Optional config filter
            limit: Maximum results
            
        Returns:
            List of checkpoints
        """
        filters = {}
        if config and config.get("checkpoint_id"):
            filters["checkpoint_id"] = config["checkpoint_id"]
        
        records = self.db.list(self.namespace, {"limit": limit, "filters": filters})
        
        checkpoints = []
        for record in records:
            try:
                checkpoint = json.loads(record["payload"])
                checkpoints.append({
                    "id": record["key"],
                    "config": json.loads(record["metadata"].get("config", "{}")),
                    "checkpoint": checkpoint,
                    "metadata": record["metadata"],
                    "created_at": record["created_at_ms"]
                })
            except json.JSONDecodeError:
                continue
        
        return sorted(checkpoints, key=lambda x: x["created_at"])
    
    def delete(self, config: Dict[str, Any]) -> bool:
        """
        Delete a checkpoint.
        
        Args:
            config: Configuration dictionary
            
        Returns:
            True if deleted
        """
        checkpoint_id = config.get("checkpoint_id")
        if checkpoint_id:
            return self.db.delete(self.namespace, checkpoint_id)
        return False
    
    def search(
        self,
        query: str,
        limit: int = 10
    ) -> List[Dict[str, Any]]:
        """
        Search checkpoints by content.
        
        Args:
            query: Search query
            limit: Maximum results
            
        Returns:
            List of matching checkpoints
        """
        hits = self.db.search_memory(
            self.namespace,
            text_query=query,
            top_k=limit
        )
        
        checkpoints = []
        for hit in hits:
            record = hit["record"]
            try:
                checkpoint = json.loads(record["payload"])
                checkpoints.append({
                    "id": record["key"],
                    "config": json.loads(record["metadata"].get("config", "{}")),
                    "checkpoint": checkpoint,
                    "score": hit["score"],
                    "created_at": record["created_at_ms"]
                })
            except json.JSONDecodeError:
                continue
        
        return checkpoints
    
    def clear(self) -> int:
        """
        Clear all checkpoints.
        
        Returns:
            Number of deleted checkpoints
        """
        checkpoints = self.list(limit=1000)
        count = 0
        for ckpt in checkpoints:
            config = {"checkpoint_id": ckpt["id"]}
            if self.delete(config):
                count += 1
        return count
    
    def get_stats(self) -> Dict[str, Any]:
        """
        Get checkpoint statistics.
        
        Returns:
            Statistics dictionary
        """
        checkpoints = self.list(limit=10000)
        
        return {
            "thread_id": self.thread_id,
            "total_checkpoints": len(checkpoints),
            "first_checkpoint": checkpoints[0]["created_at"] if checkpoints else None,
            "last_checkpoint": checkpoints[-1]["created_at"] if checkpoints else None
        }
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB CheckpointStore for LangGraph."""
    
    store = VantaDBCheckpointStore(thread_id="demo-workflow")
    
    print("📝 Storing checkpoints...")
    
    # Store checkpoints
    config1 = {"thread_id": "demo-workflow", "checkpoint_id": "ckpt-001"}
    checkpoint1 = {
        "state": {"messages": ["Hello"], "step": 1},
        "metadata": {"node": "start"}
    }
    store.put(config1, checkpoint1, metadata={"version": "1.0"})
    
    config2 = {"thread_id": "demo-workflow", "checkpoint_id": "ckpt-002"}
    checkpoint2 = {
        "state": {"messages": ["Hello", "How can I help?"], "step": 2},
        "metadata": {"node": "agent"}
    }
    store.put(config2, checkpoint2, metadata={"version": "1.0"})
    
    config3 = {"thread_id": "demo-workflow", "checkpoint_id": "ckpt-003"}
    checkpoint3 = {
        "state": {"messages": ["Hello", "How can I help?", "I need help with Python"], "step": 3},
        "metadata": {"node": "agent"}
    }
    store.put(config3, checkpoint3, metadata={"version": "1.0"})
    
    print("\n📊 Checkpoint statistics:")
    stats = store.get_stats()
    print(f"  Thread ID: {stats['thread_id']}")
    print(f"  Total checkpoints: {stats['total_checkpoints']}")
    
    print("\n🔍 Listing checkpoints:")
    checkpoints = store.list()
    for ckpt in checkpoints:
        print(f"  [{ckpt['id']}] Step: {ckpt['checkpoint']['state']['step']}")
        print(f"      Messages: {len(ckpt['checkpoint']['state']['messages'])}")
    
    print("\n🔍 Searching for 'Python'...")
    results = store.search("Python")
    for result in results:
        print(f"  Score: {result['score']:.3f}")
        print(f"  Step: {result['checkpoint']['state']['step']}")
        print(f"  Messages: {result['checkpoint']['state']['messages'][-1]}")
    
    print("\n🧹 Cleaning up...")
    store.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
