#!/bin/bash

# Quick test script - just checks if server responds
# Useful for CI/CD or quick validation

if curl -s -f http://localhost:3000/health > /dev/null 2>&1; then
    echo "✅ Server is running"
    exit 0
else
    echo "❌ Server is not running"
    exit 1
fi
