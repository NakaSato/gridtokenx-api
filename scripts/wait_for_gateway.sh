#!/bin/bash
echo "Waiting for API Gateway to be ready..."
RETRIES=30
while [ $RETRIES -gt 0 ]; do
    if curl -s http://localhost:4000/health > /dev/null; then
        echo "API Gateway is ready!"
        exit 0
    fi
    echo -n "."
    sleep 5
    RETRIES=$((RETRIES-1))
done

echo "API Gateway failed to start!"
cat api.log
exit 1
