#!/bin/bash
set -e

# Roanoke Engine Deployment Script
# Run this script to build and package the game for deployment

DEPLOY_DIR="./deploy/roanoke_game"
BINARY_NAME="roanoke_game"

echo "=== Roanoke Engine Deployment ==="
echo ""

# Step 1: Clean previous deployment
echo "[1/6] Cleaning previous deployment..."
rm -rf ./deploy

# Step 2: Build release binary
echo "[2/6] Building release binary (this may take a while)..."
cargo build --release

# Step 3: Create deployment directory structure
echo "[3/6] Creating deployment directory structure..."
mkdir -p "$DEPLOY_DIR/assets/shaders"
mkdir -p "$DEPLOY_DIR/assets/ui/loading"
mkdir -p "$DEPLOY_DIR/trees"

# Step 4: Copy binary
echo "[4/6] Copying binary..."
cp "target/release/$BINARY_NAME" "$DEPLOY_DIR/"

# Step 5: Copy assets
echo "[5/6] Copying assets..."
# Shaders
cp -r assets/shaders/*.wgsl "$DEPLOY_DIR/assets/shaders/" 2>/dev/null || echo "  Warning: No shader files found"
# UI assets
cp -r assets/ui/* "$DEPLOY_DIR/assets/ui/" 2>/dev/null || echo "  Warning: No UI assets found"
# Root textures
cp assets/*.jpg "$DEPLOY_DIR/assets/" 2>/dev/null || echo "  Warning: No root textures found"
# Tree models
cp -r trees/* "$DEPLOY_DIR/trees/" 2>/dev/null || echo "  Warning: No tree models found"

# Step 6: Optional - strip binary to reduce size
echo "[6/6] Stripping binary (optional optimization)..."
strip "$DEPLOY_DIR/$BINARY_NAME" 2>/dev/null || echo "  Warning: strip command not available"

# Summary
echo ""
echo "=== Deployment Complete ==="
echo "Location: $DEPLOY_DIR"
echo ""
echo "Contents:"
du -sh "$DEPLOY_DIR"/* 2>/dev/null || ls -la "$DEPLOY_DIR"
echo ""
echo "To run the game:"
echo "  cd $DEPLOY_DIR && ./$BINARY_NAME"
echo ""
echo "To create a distributable archive:"
echo "  tar -czvf roanoke_game.tar.gz -C ./deploy roanoke_game"
