#!/usr/bin/env python3
"""
Script to create and initialize a VantaDB namespace
Useful for setting up isolated memory spaces for different agents, sessions, or projects
"""

import sys
import os
import argparse

try:
    import vantadb as vantadb
except ImportError:
    print("❌ VantaDB Python SDK not found. Install with: pip install vantadb")
    sys.exit(1)

def create_namespace(db_path, namespace, description=None):
    """Create and initialize a namespace"""
    print(f"🔧 Creating namespace: {namespace}")
    print(f"   Database path: {db_path}")
    
    try:
        # Open VantaDB
        db = vantadb.VantaDB(db_path, memory_limit_bytes=256_000_000)
        
        # Add a system record to initialize the namespace
        system_key = f"{namespace}/_system_info"
        system_content = f"Namespace: {namespace}\n"
        if description:
            system_content += f"Description: {description}\n"
        system_content += f"Created: {__import__('datetime').datetime.now().isoformat()}\n"
        
        db.put(
            namespace,
            system_key,
            system_content,
            metadata={"type": "system", "namespace": namespace}
        )
        
        print(f"✅ Namespace '{namespace}' created successfully")
        print(f"   System record: {system_key}")
        
        # Close database
        db.flush()
        db.close()
        
        return True
        
    except Exception as e:
        print(f"❌ Error creating namespace: {e}")
        return False

def list_namespaces(db_path):
    """List all namespaces in the database"""
    print(f"📋 Listing namespaces in: {db_path}")
    
    try:
        db = vantadb.VantaDB(db_path, memory_limit_bytes=256_000_000)
        namespaces = db.list_namespaces()
        
        if not namespaces:
            print("   No namespaces found")
        else:
            print(f"   Found {len(namespaces)} namespace(s):")
            for ns in namespaces:
                print(f"   - {ns}")
        
        db.flush()
        db.close()
        
        return True
        
    except Exception as e:
        print(f"❌ Error listing namespaces: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="Manage VantaDB namespaces")
    parser.add_argument("--path", default="~/.vantadb", help="Database path (default: ~/.vantadb)")
    
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # Create command
    create_parser = subparsers.add_parser("create", help="Create a new namespace")
    create_parser.add_argument("namespace", help="Namespace name")
    create_parser.add_argument("--description", help="Namespace description")
    
    # List command
    list_parser = subparsers.add_parser("list", help="List all namespaces")
    
    args = parser.parse_args()
    
    db_path = os.path.expanduser(args.path)
    
    if args.command == "create":
        success = create_namespace(db_path, args.namespace, args.description)
        sys.exit(0 if success else 1)
    elif args.command == "list":
        success = list_namespaces(db_path)
        sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
