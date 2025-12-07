#!/bin/bash
set -e

echo "=== Initializing Docusaurus Documentation Site ==="

# Create docs directory
mkdir -p docs

cd docs

# Initialize Docusaurus
npx create-docusaurus@latest . classic --typescript

echo "=== Docusaurus initialized ==="
echo "Run 'cd docs && npm start' to start the dev server"

