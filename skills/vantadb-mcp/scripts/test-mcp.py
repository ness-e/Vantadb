#!/usr/bin/env python3
"""
Test script for VantaDB MCP server
Tests basic MCP functionality to ensure the server is working correctly
"""

import subprocess
import json
import sys
import os

def send_rpc_request(request):
    """Send a JSON-RPC request to the MCP server"""
    process = subprocess.Popen(
        ["vanta-server", "--mcp", "--path", os.path.expanduser("~/.vantadb")],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    stdout, stderr = process.communicate(input=json.dumps(request) + "\n")
    
    if process.returncode != 0:
        print(f"❌ Server error: {stderr}")
        return None
    
    try:
        return json.loads(stdout)
    except json.JSONDecodeError:
        print(f"❌ Invalid JSON response: {stdout}")
        return None

def test_initialize():
    """Test initialize method"""
    print("🔍 Testing initialize...")
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    }
    
    response = send_rpc_request(request)
    if response and "result" in response:
        print("✅ Initialize successful")
        print(f"   Server: {response['result']['serverInfo']['name']}")
        print(f"   Version: {response['result']['serverInfo']['version']}")
        return True
    return False

def test_tools_list():
    """Test tools/list method"""
    print("\n🔍 Testing tools/list...")
    request = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    }
    
    response = send_rpc_request(request)
    if response and "result" in response:
        tools = response["result"].get("tools", [])
        print(f"✅ Found {len(tools)} tools")
        for tool in tools[:5]:  # Show first 5
            print(f"   - {tool['name']}")
        if len(tools) > 5:
            print(f"   ... and {len(tools) - 5} more")
        return True
    return False

def test_resources_list():
    """Test resources/list method"""
    print("\n🔍 Testing resources/list...")
    request = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list"
    }
    
    response = send_rpc_request(request)
    if response and "result" in response:
        resources = response["result"].get("resources", [])
        print(f"✅ Found {len(resources)} resources")
        for resource in resources:
            print(f"   - {resource['uri']}")
        return True
    return False

def test_prompts_list():
    """Test prompts/list method"""
    print("\n🔍 Testing prompts/list...")
    request = {
        "jsonrpc": "2.0",
        "id": 4,
        "method": "prompts/list"
    }
    
    response = send_rpc_request(request)
    if response and "result" in response:
        prompts = response["result"].get("prompts", [])
        print(f"✅ Found {len(prompts)} prompts")
        for prompt in prompts:
            print(f"   - {prompt['name']}")
        return True
    return False

def main():
    """Run all tests"""
    print("🧪 Testing VantaDB MCP Server")
    print("=" * 50)
    
    tests = [
        test_initialize,
        test_tools_list,
        test_resources_list,
        test_prompts_list
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test failed with exception: {e}")
            failed += 1
    
    print("\n" + "=" * 50)
    print(f"📊 Results: {passed} passed, {failed} failed")
    
    if failed == 0:
        print("✅ All tests passed!")
        sys.exit(0)
    else:
        print("❌ Some tests failed")
        sys.exit(1)

if __name__ == "__main__":
    main()
