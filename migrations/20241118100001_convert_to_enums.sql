-- Convert VARCHAR columns to proper ENUM types
-- Created: November 18, 2024

-- Create order_side enum if it doesn't exist
DO $$ BEGIN
    CREATE TYPE order_side AS ENUM ('Buy', 'Sell');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create order_status enum if it doesn't exist (with capital letters to match code)
DO $$ BEGIN
    DROP TYPE IF EXISTS order_status CASCADE;
    CREATE TYPE order_status AS ENUM ('Pending', 'Active', 'PartiallyFilled', 'Filled', 'Settled', 'Cancelled', 'Expired');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Drop existing default before converting
ALTER TABLE trading_orders ALTER COLUMN status DROP DEFAULT;

-- Convert trading_orders.status from VARCHAR to order_status enum
ALTER TABLE trading_orders 
    ALTER COLUMN status TYPE order_status 
    USING (
        CASE 
            WHEN status = 'pending' THEN 'Pending'::order_status
            WHEN status = 'active' THEN 'Active'::order_status
            WHEN status = 'partially_filled' THEN 'PartiallyFilled'::order_status
            WHEN status = 'filled' THEN 'Filled'::order_status
            WHEN status = 'settled' THEN 'Settled'::order_status
            WHEN status = 'cancelled' THEN 'Cancelled'::order_status
            WHEN status = 'expired' THEN 'Expired'::order_status
            ELSE 'Pending'::order_status
        END
    );

-- Convert trading_orders.side from VARCHAR to order_side enum
ALTER TABLE trading_orders 
    ALTER COLUMN side TYPE order_side 
    USING (
        CASE 
            WHEN LOWER(side) = 'buy' THEN 'Buy'::order_side
            WHEN LOWER(side) = 'sell' THEN 'Sell'::order_side
            ELSE 'Buy'::order_side
        END
    );

-- Update default value for status
ALTER TABLE trading_orders ALTER COLUMN status SET DEFAULT 'Pending'::order_status;

-- Drop old CHECK constraints
ALTER TABLE trading_orders DROP CONSTRAINT IF EXISTS chk_order_status;
ALTER TABLE trading_orders DROP CONSTRAINT IF EXISTS chk_order_type;
