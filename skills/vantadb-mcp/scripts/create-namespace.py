#!/usr/bin/env python3
"""
Script to create and initialize a VantaDB namespace
Useful for setting up isolated memory spaces for different agents, sessions, or projects
"""

import sys
import os
import argparse

try:
    import vantadb_py as vantadb
except ImportError:
    print("❌ VantaDB Python SDK not found. Install with: pip install vantadb-py")
    sys.exit(1)

def create_namespace(db_path, namespace, description=None):
    """Create and initialize a namespace"""
    print("[SETUP] Creating namespace: %s" % namespace)
    print("        Database path: %s" % db_path)
    
    try:
        # Open VantaDB
        db = vantadb.VantaDB(db_path, memory_limit_bytes=256_000_000)
        
        # Add a system record to initialize the namespace
        system_key = "%s/_system_info" % namespace
        system_content = "Namespace: %s\n" % namespace
        if description:
            system_content += "Description: %s\n" % description
        system_content += "Created: %s\n" % __import__('datetime').datetime.now().isoformat()
        
        db.put(
            namespace,
            system_key,
            system_content,
            metadata={"type": "system", "namespace": namespace}
        )
        
        print("[OK] Namespace '%s' created successfully" % namespace)
        print("     System record: %s" % system_key)
        
        # Close database
        db.flush()
        db.close()
        
        return True
        
    except Exception as e:
        print("[ERROR] Error creating namespace: %s" % e)
        return False

# list_namespaces is not exposed in Python SDK; use `vanta-cli namespace list` instead.

def main():
    parser = argparse.ArgumentParser(description="Manage VantaDB namespaces")
    parser.add_argument("--path", default="~/.vantadb", help="Database path (default: ~/.vantadb)")
    
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # Create command
    create_parser = subparsers.add_parser("create", help="Create a new namespace")
    create_parser.add_argument("namespace", help="Namespace name")
    create_parser.add_argument("--description", help="Namespace description")
    
    args = parser.parse_args()
    
    db_path = os.path.expanduser(args.path)
    
    if args.command == "create":
        success = create_namespace(db_path, args.namespace, args.description)
        sys.exit(0 if success else 1)
    else:
        print("Use `vanta-cli namespace list` to list namespaces")
        sys.exit(1)

if __name__ == "__main__":
    main()
