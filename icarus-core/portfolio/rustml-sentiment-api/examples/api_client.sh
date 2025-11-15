#!/bin/bash

# Example API client for RustML Sentiment API
# Demonstrates how to interact with the API using curl

API_URL="http://localhost:3000"

echo "🦀 RustML Sentiment API - Client Examples"
echo "=========================================="
echo ""

# Check if server is running
echo "🔍 Checking server health..."
curl -s "${API_URL}/health" | jq '.'
echo ""

# Example 1: Sentiment Analysis
echo "📊 Example 1: Sentiment Analysis"
echo "─────────────────────────────────"
curl -s -X POST "${API_URL}/api/v1/sentiment" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "This product is absolutely amazing! Best purchase ever!"
  }' | jq '.'
echo ""

# Example 2: Text Classification
echo "📁 Example 2: Text Classification"
echo "─────────────────────────────────"
curl -s -X POST "${API_URL}/api/v1/classify" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Machine learning and AI are transforming the technology industry"
  }' | jq '.'
echo ""

# Example 3: Combined Analysis
echo "🔬 Example 3: Combined Analysis"
echo "─────────────────────────────────"
curl -s -X POST "${API_URL}/api/v1/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "The new AI-powered software is revolutionary and impressive!"
  }' | jq '.'
echo ""

# Example 4: Batch Analysis
echo "🔄 Example 4: Batch Analysis"
echo "─────────────────────────────────"
curl -s -X POST "${API_URL}/api/v1/batch" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      "Excellent product, highly recommended!",
      "Terrible experience, very disappointed.",
      "It works as expected, nothing special."
    ]
  }' | jq '.'
echo ""

# Example 5: Trend Analysis
echo "📈 Example 5: Trend Analysis"
echo "─────────────────────────────────"
curl -s -X POST "${API_URL}/api/v1/trend" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      "Great!",
      "Wonderful!",
      "Not bad.",
      "Terrible!",
      "Amazing!"
    ]
  }' | jq '.'
echo ""

# Get API Statistics
echo "📊 API Statistics"
echo "─────────────────────────────────"
curl -s "${API_URL}/stats" | jq '.'
echo ""

echo "✅ All examples completed!"
