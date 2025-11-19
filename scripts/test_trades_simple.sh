#!/bin/bash

# Extract database URL from .env
DATABASE_URL=$(grep DATABASE_URL .env | cut -d'=' -f2-)

echo "Testing trades table existence..."
echo "Database URL: $DATABASE_URL"

# Try to create trades table directly using SQL
cat << 'EOF' | psql "$DATABASE_URL" 2>/dev/null || echo "psql not available, trying alternative..."

-- Check if trades table exists
SELECT EXISTS (
    SELECT FROM information_schema.tables 
    WHERE table_schema = 'current_schema()' 
    AND table_name = 'trades'
) as trades_exists;

-- If trades table doesn't exist, create it
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'current_schema()' 
        AND table_name = 'trades'
    ) THEN
        RAISE NOTICE 'Creating trades table...';
        
        CREATE TABLE trades (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            buy_order_id UUID NOT NULL REFERENCES trading_orders(id) ON DELETE CASCADE,
            sell_order_id UUID NOT NULL REFERENCES trading_orders(id) ON DELETE CASCADE,
            buyer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            seller_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            quantity NUMERIC(20, 8) NOT NULL,
            price NUMERIC(20, 8) NOT NULL,
            total_value NUMERIC(20, 8) GENERATED ALWAYS AS (quantity * price) STORED,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            status VARCHAR(20) NOT NULL DEFAULT 'completed',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );
        
        CREATE INDEX idx_trades_buyer_id ON trades(buyer_id);
        CREATE INDEX idx_trades_seller_id ON trades(seller_id);
        CREATE INDEX idx_trades_executed_at ON trades(executed_at);
        
        RAISE NOTICE 'Trades table created successfully';
    ELSE
        RAISE NOTICE 'Trades table already exists';
    END IF;
END $$;

-- Show table structure
SELECT column_name, data_type, is_nullable 
FROM information_schema.columns 
WHERE table_name = 'trades' 
ORDER BY ordinal_position;

-- Count records
SELECT COUNT(*) as trade_count FROM trades;

EOF

if [ $? -ne 0 ]; then
    echo "psql failed, trying with docker if available..."
    docker run --rm postgres:15 psql "$DATABASE_URL" << 'EOF'
SELECT EXISTS (
    SELECT FROM information_schema.tables 
    WHERE table_schema = 'current_schema()' 
    AND table_name = 'trades'
) as trades_exists;
EOF
fi
