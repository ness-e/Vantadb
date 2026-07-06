"""
VantaDB Integration for AutoGen

This example demonstrates how to use VantaDB as a persistent memory backend
for AutoGen conversational agents. VantaDB provides namespace-scoped memory
with hybrid vector and text search for context-aware conversations.
"""

import vantadb_py as vantadb
from typing import List, Dict, Any, Optional
import json
import os

DB_PATH = "./autogen_vantadb_db"

class VantaDBAutoGenMemory:
    """
    VantaDB-based memory backend for AutoGen agents.
    
    Provides persistent conversation memory with:
    - Thread-scoped isolation
    - Message-level storage
    - Hybrid search for context retrieval
    - Metadata for message types and roles
    """
    
    def __init__(self, db_path: str = DB_PATH, thread_id: str = "default"):
        """
        Initialize VantaDB memory for AutoGen.
        
        Args:
            db_path: Path to VantaDB database
            thread_id: Conversation thread identifier
        """
        self.db = vantadb.VantaDB(db_path, memory_limit_bytes=512_000_000)
        self.thread_id = thread_id
        self.namespace = f"autogen/thread-{thread_id}"
        
    def add_message(
        self,
        role: str,
        content: str,
        message_type: str = "chat",
        metadata: Optional[Dict[str, Any]] = None,
        vector: Optional[List[float]] = None
    ) -> Dict[str, Any]:
        """
        Add a message to conversation memory.
        
        Args:
            role: Message role (user, assistant, system)
            content: Message content
            message_type: Message type (chat, system, function_call)
            metadata: Optional metadata
            vector: Optional embedding vector
            
        Returns:
            Stored message record
        """
        import uuid
        key = f"{role}-{uuid.uuid4().hex[:8]}"
        
        meta = metadata or {}
        meta.update({
            "role": role,
            "type": message_type,
            "thread_id": self.thread_id
        })
        
        record = self.db.put(
            self.namespace,
            key,
            content,
            metadata=meta,
            vector=vector
        )
        
        return {
            "id": key,
            "role": role,
            "content": content,
            "type": message_type,
            "metadata": meta,
            "created_at": record["created_at_ms"]
        }
    
    def get_message(self, message_id: str) -> Optional[Dict[str, Any]]:
        """
        Retrieve a message by ID.
        
        Args:
            message_id: Message identifier
            
        Returns:
            Message record or None
        """
        record = self.db.get(self.namespace, message_id)
        if record:
            return {
                "id": record["key"],
                "role": record["metadata"].get("role"),
                "content": record["payload"],
                "type": record["metadata"].get("type"),
                "metadata": record["metadata"],
                "created_at": record["created_at_ms"]
            }
        return None
    
    def search_context(
        self,
        query: str,
        query_vector: Optional[List[float]] = None,
        role_filter: Optional[str] = None,
        limit: int = 10
    ) -> List[Dict[str, Any]]:
        """
        Search conversation context with hybrid search.
        
        Args:
            query: Search query
            query_vector: Optional vector for semantic search
            role_filter: Optional role filter (user, assistant, system)
            limit: Maximum results
            
        Returns:
            List of matching messages
        """
        filters = {}
        if role_filter:
            filters["role"] = role_filter
        
        hits = self.db.search_memory(
            self.namespace,
            query_vector=query_vector,
            text_query=query,
            top_k=limit,
            filters=filters
        )
        
        results = []
        for hit in hits:
            metadata = hit.metadata
            results.append({
                "id": hit.key,
                "role": metadata.get("role"),
                "content": hit.payload,
                "type": metadata.get("type"),
                "score": hit.score,
                "created_at": hit.created_at_ms
            })
        
        return results
    
    def get_conversation_history(
        self,
        role_filter: Optional[str] = None,
        limit: int = 100
    ) -> List[Dict[str, Any]]:
        """
        Get conversation history.
        
        Args:
            role_filter: Optional role filter
            limit: Maximum messages to retrieve
            
        Returns:
            List of messages in chronological order
        """
        filters = {"role": role_filter} if role_filter else {}
        records = self.db.list(self.namespace, {"limit": limit, "filters": filters})
        
        return [
            {
                "id": r["key"],
                "role": r["metadata"].get("role"),
                "content": r["payload"],
                "type": r["metadata"].get("type"),
                "created_at": r["created_at_ms"]
            }
            for r in sorted(records, key=lambda x: x["created_at_ms"])
        ]
    
    def delete_message(self, message_id: str) -> bool:
        """
        Delete a message.
        
        Args:
            message_id: Message identifier
            
        Returns:
            True if deleted
        """
        return self.db.delete(self.namespace, message_id)
    
    def clear_conversation(self) -> int:
        """
        Clear all messages in the conversation.
        
        Returns:
            Number of deleted messages
        """
        messages = self.get_conversation_history(limit=1000)
        count = 0
        for msg in messages:
            if self.delete_message(msg["id"]):
                count += 1
        return count
    
    def get_summary(self) -> Dict[str, Any]:
        """
        Get conversation summary.
        
        Returns:
            Summary statistics
        """
        messages = self.get_conversation_history()
        
        role_counts = {}
        for msg in messages:
            role = msg["role"]
            role_counts[role] = role_counts.get(role, 0) + 1
        
        return {
            "thread_id": self.thread_id,
            "total_messages": len(messages),
            "role_counts": role_counts,
            "first_message": messages[0]["created_at"] if messages else None,
            "last_message": messages[-1]["created_at"] if messages else None
        }
    
    def close(self):
        """Close the database connection."""
        self.db.flush()
        self.db.close()


def main():
    """Demonstrate VantaDB memory for AutoGen."""
    
    memory = VantaDBAutoGenMemory(thread_id="demo-conversation")
    
    print("📝 Simulating conversation...")
    
    # Add conversation messages
    memory.add_message(
        "user",
        "I need help with a Python script for data analysis",
        message_type="chat"
    )
    
    memory.add_message(
        "assistant",
        "I can help you with that. What kind of data analysis do you need?",
        message_type="chat"
    )
    
    memory.add_message(
        "user",
        "I need to analyze sales data from a CSV file using pandas",
        message_type="chat"
    )
    
    memory.add_message(
        "assistant",
        "Great! I'll help you create a script to load and analyze CSV data with pandas",
        message_type="chat"
    )
    
    memory.add_message(
        "system",
        "User has Python 3.11 and pandas 2.0 installed",
        message_type="system"
    )
    
    print("\n📊 Conversation summary:")
    summary = memory.get_summary()
    print(f"  Thread ID: {summary['thread_id']}")
    print(f"  Total messages: {summary['total_messages']}")
    print(f"  Role distribution: {summary['role_counts']}")
    
    print("\n🔍 Searching for 'pandas'...")
    results = memory.search_context("pandas")
    for result in results:
        print(f"  [{result['role']}] Score: {result['score']:.3f}")
        print(f"  {result['content']}")
    
    print("\n📜 Conversation history:")
    history = memory.get_conversation_history()
    for msg in history:
        print(f"  [{msg['role']}] {msg['content'][:60]}...")
    
    print("\n🧹 Cleaning up...")
    memory.close()
    if os.path.exists(DB_PATH):
        import shutil
        shutil.rmtree(DB_PATH)
        print("Database cleaned up.")


if __name__ == "__main__":
    main()
