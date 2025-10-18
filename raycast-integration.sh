#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title German-English Dictionary
# @raycast.mode fullOutput
# @raycast.packageName dictv

# Optional parameters:
# @raycast.icon 📖
# @raycast.argument1 { "type": "text", "placeholder": "Word to look up" }
# @raycast.argument2 { "type": "dropdown", "placeholder": "Direction", "data": [{"title": "German → English", "value": "de-en"}, {"title": "English → German", "value": "en-de"}], "optional": true }

# Documentation:
# @raycast.description Look up German-English translations with fuzzy search
# @raycast.author Your Name
# @raycast.authorURL https://github.com/yourusername

query="$1"
lang="${2:-de-en}"  # Default to German → English

# Validate input
if [ -z "$query" ]; then
    echo "❌ Error: Please provide a word to look up"
    exit 1
fi

# Check if server is running
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "❌ Error: dictv server is not running"
    echo ""
    echo "Please start the server with:"
    echo "  dictv serve --port 3000"
    exit 1
fi

# Query the API
response=$(curl -s "http://localhost:3000/search?q=$query&mode=fuzzy&lang=$lang&max_distance=2&limit=10")

# Check if request was successful
if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to connect to dictv server"
    exit 1
fi

# Parse the response
total_results=$(echo "$response" | jq -r '.total_results')
query_time=$(echo "$response" | jq -r '.query_time_ms')

# Display header
if [ "$lang" = "de-en" ]; then
    echo "📖 German → English: \"$query\""
else
    echo "📖 English → German: \"$query\""
fi
echo "⏱️  Query time: ${query_time}ms"
echo ""

# Check if any results found
if [ "$total_results" -eq 0 ]; then
    echo "❌ No results found for \"$query\""
    echo ""
    echo "💡 Try:"
    echo "  • Checking your spelling"
    echo "  • Using a different search term"
    echo "  • Switching the language direction"
    exit 0
fi

echo "📚 Found $total_results result(s):"
echo ""

# Format and display results
echo "$response" | jq -r '.results[] |
    if .edit_distance then
        "• \(.word) [distance: \(.edit_distance)]\n  → \(.definition)\n"
    else
        "• \(.word)\n  → \(.definition)\n"
    end'

# Display footer with tips
if [ "$total_results" -ge 10 ]; then
    echo "💡 Showing top 10 results. Refine your search for more specific results."
fi
