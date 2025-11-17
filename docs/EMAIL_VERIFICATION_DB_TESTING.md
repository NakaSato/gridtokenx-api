# Email Verification Database Testing Guide

## Quick Database Verification Commands

These commands can be used to verify the email verification schema is working correctly.

---

## 1. Verify Schema

### Check all email verification columns exist
```sql
SELECT 
    column_name,
    data_type,
    is_nullable,
    column_default
FROM information_schema.columns 
WHERE table_name = 'users' 
AND column_name LIKE 'email%'
ORDER BY ordinal_position;
```

### View full users table structure
```bash
docker exec p2p-postgres psql -U p2p_user -d p2p_energy_trading -c "\d users"
```

---

## 2. Verify Indexes

### List all indexes on users table
```sql
SELECT 
    indexname,
    indexdef
FROM pg_indexes 
WHERE tablename = 'users'
ORDER BY indexname;
```

### Check email verification specific indexes
```sql
SELECT 
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes 
WHERE tablename = 'users' 
AND (indexname LIKE '%email%' OR indexname LIKE '%verif%');
```

---

## 3. Test Data Operations

### Create a test user (unverified)
```sql
INSERT INTO users (
    username, 
    email, 
    password_hash, 
    role, 
    first_name, 
    last_name,
    email_verified
) VALUES (
    'test_user_' || floor(random() * 1000)::text,
    'test' || floor(random() * 1000)::text || '@example.com',
    '$2b$12$test.hash.value',
    'user',
    'Test',
    'User',
    false
) RETURNING id, username, email, email_verified;
```

### Simulate sending verification email
```sql
UPDATE users 
SET 
    email_verification_token = encode(gen_random_bytes(32), 'hex'),
    email_verification_sent_at = NOW(),
    email_verification_expires_at = NOW() + INTERVAL '24 hours'
WHERE username = 'test_user_123'
RETURNING 
    username,
    email_verification_sent_at,
    email_verification_expires_at;
```

### Simulate successful email verification
```sql
UPDATE users 
SET 
    email_verified = true,
    email_verified_at = NOW(),
    email_verification_token = NULL,
    email_verification_sent_at = NULL,
    email_verification_expires_at = NULL
WHERE username = 'test_user_123'
RETURNING 
    username,
    email_verified,
    email_verified_at;
```

---

## 4. Query Examples

### Find all unverified users
```sql
SELECT 
    id,
    username,
    email,
    email_verified,
    created_at
FROM users 
WHERE email_verified = false
ORDER BY created_at DESC;
```

### Find users with pending verification tokens
```sql
SELECT 
    id,
    username,
    email,
    email_verification_sent_at,
    email_verification_expires_at,
    CASE 
        WHEN email_verification_expires_at < NOW() THEN 'EXPIRED'
        ELSE 'VALID'
    END as token_status
FROM users 
WHERE email_verification_token IS NOT NULL
ORDER BY email_verification_sent_at DESC;
```

### Find users who verified recently (last 24 hours)
```sql
SELECT 
    id,
    username,
    email,
    email_verified_at,
    created_at,
    email_verified_at - created_at as verification_time
FROM users 
WHERE email_verified_at IS NOT NULL
AND email_verified_at > NOW() - INTERVAL '24 hours'
ORDER BY email_verified_at DESC;
```

### Statistics on verification rates
```sql
SELECT 
    COUNT(*) as total_users,
    COUNT(*) FILTER (WHERE email_verified = true) as verified_users,
    COUNT(*) FILTER (WHERE email_verified = false) as unverified_users,
    ROUND(
        100.0 * COUNT(*) FILTER (WHERE email_verified = true) / 
        NULLIF(COUNT(*), 0), 
        2
    ) as verification_rate_percent
FROM users;
```

---

## 5. Performance Testing

### Check index usage on verification token lookup
```sql
EXPLAIN ANALYZE
SELECT * FROM users 
WHERE email_verification_token = 'sample_token_hash'
AND is_active = true;
```

### Check index usage on email verification status
```sql
EXPLAIN ANALYZE
SELECT * FROM users 
WHERE email_verified = false
AND email_verification_expires_at < NOW();
```

---

## 6. Data Cleanup

### Remove expired verification tokens (maintenance query)
```sql
UPDATE users 
SET 
    email_verification_token = NULL,
    email_verification_sent_at = NULL,
    email_verification_expires_at = NULL
WHERE email_verification_expires_at < NOW()
AND email_verification_token IS NOT NULL
RETURNING username, email;
```

### Delete test users
```sql
DELETE FROM users 
WHERE username LIKE 'test_user_%'
RETURNING username;
```

---

## 7. Migration Verification

### Check migration status
```sql
SELECT 
    version,
    description,
    success,
    installed_on,
    execution_time
FROM _sqlx_migrations 
WHERE version >= 20241102000000
ORDER BY version;
```

### Verify migration checksum
```sql
SELECT 
    version,
    description,
    encode(checksum, 'hex') as checksum_hex
FROM _sqlx_migrations 
WHERE version = 20241102000002;
```

---

## 8. User Activities Logging Test

### Insert test activity log
```sql
INSERT INTO user_activities (
    user_id,
    action,
    details,
    ip_address,
    user_agent
) 
SELECT 
    id,
    'email_verification_sent',
    jsonb_build_object(
        'email', email,
        'timestamp', NOW()
    ),
    '127.0.0.1',
    'Test User Agent'
FROM users 
WHERE username = 'test_user_123'
RETURNING *;
```

### Query recent verification activities
```sql
SELECT 
    ua.action,
    u.username,
    u.email,
    ua.details,
    ua.created_at
FROM user_activities ua
JOIN users u ON ua.user_id = u.id
WHERE ua.action IN (
    'email_verification_sent',
    'email_verified',
    'email_verification_failed'
)
ORDER BY ua.created_at DESC
LIMIT 10;
```

---

## 9. Constraints and Validation

### Test email_verified default value
```sql
INSERT INTO users (username, email, password_hash, role, first_name, last_name)
VALUES ('default_test', 'default@test.com', '$2b$12$hash', 'user', 'Default', 'Test')
RETURNING username, email_verified;
-- Should return email_verified = false
```

### Test NOT NULL constraint on email_verified
```sql
-- This should FAIL
INSERT INTO users (username, email, password_hash, role, first_name, last_name, email_verified)
VALUES ('null_test', 'null@test.com', '$2b$12$hash', 'user', 'Null', 'Test', NULL);
-- Expected: ERROR: null value in column "email_verified" violates not-null constraint
```

---

## 10. Docker Commands

### Connect to PostgreSQL in Docker
```bash
docker exec -it p2p-postgres psql -U p2p_user -d p2p_energy_trading
```

### Run SQL file in Docker
```bash
docker exec -i p2p-postgres psql -U p2p_user -d p2p_energy_trading < /path/to/file.sql
```

### Backup database with new schema
```bash
docker exec p2p-postgres pg_dump -U p2p_user -d p2p_energy_trading > backup_with_email_verification.sql
```

---

## Expected Results Summary

✅ **5 new columns** in users table
✅ **3 new indexes** created
✅ **All columns** have NOT NULL or proper defaults
✅ **Comments** added to all new columns
✅ **Migration** recorded in _sqlx_migrations
✅ **No performance degradation** on existing queries
✅ **Partial indexes** working correctly

---

## Common Issues and Solutions

### Issue: Migration not showing as applied
**Solution**: 
```sql
SELECT * FROM _sqlx_migrations WHERE version = 20241102000002;
```
If missing, manually insert or re-run migration.

### Issue: Index not being used
**Solution**: 
```sql
ANALYZE users;
VACUUM ANALYZE users;
```
Update statistics for query planner.

### Issue: Can't drop column (in use)
**Solution**: Check for views or triggers:
```sql
SELECT * FROM information_schema.view_column_usage WHERE table_name = 'users';
```

---

**Last Updated**: November 2, 2025
**Database Version**: PostgreSQL 16
**Migration Version**: 20241102000002
