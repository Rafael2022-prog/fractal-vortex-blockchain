#!/bin/bash
# Fractal-Vortex Chain Genesis Start Script

echo "🌀 Starting Fractal-Vortex Genesis Network..."

# Start validator nodes
for i in {0..3}; do
    echo "🚀 Starting Node $i..."
    cargo run -- start --node-id $i &
    sleep 2
done

echo "✅ All nodes started!"
echo "📊 Check logs in node-*.log files"
