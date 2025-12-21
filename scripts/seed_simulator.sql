INSERT INTO users (id, email, username, password_hash, first_name, last_name, role, wallet_address, is_active, email_verified, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    'sim_user@example.com',
    'sim_user',
    '$argon2id$v=19$m=19456,t=2,p=1$DummyHashForTest$DummyHashForTest',
    'Sim',
    'User',
    'prosumer',
    'Fa3FHRjY1QxE9mc2NhJoGcSMsRuV83eBYVUEdt5Py7Xv',
    true,
    true,
    NOW(),
    NOW()
) ON CONFLICT (email) DO NOTHING;

INSERT INTO meter_registry (id, meter_serial, user_id, meter_type, meter_key_hash, verification_method, verification_status, installation_date, location_address, created_at, updated_at)
SELECT
    gen_random_uuid(),
    'SIM-METER-001',
    id,
    'smart_meter',
    'dummy_hash',
    'serial',
    'verified',
    NOW(),
    'Bangkok',
    NOW(),
    NOW()
FROM users WHERE email = 'sim_user@example.com'
ON CONFLICT (meter_serial) DO NOTHING;
