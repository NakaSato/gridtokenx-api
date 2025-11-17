# Email Verification - Quick Reference

## 🎯 Quick Start

### 1. Environment Setup
```env
# Required for email verification
EMAIL_ENABLED=true
EMAIL_VERIFICATION_REQUIRED=true
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@gridtokenx.com
```

### 2. Apply Migration
```bash
cd api-gateway
sqlx migrate run
```

### 3. Test It Works
```bash
cargo test --test email_verification_test
```

---

## 🔧 API Endpoints

### Register (Sends Verification Email)
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "username": "user123"
}

→ 201 Created
{
  "message": "Registration successful. Please check your email...",
  "user": { "id": "...", "email": "...", "username": "..." },
  "verification_required": true
}
```

### Verify Email
```http
GET /api/auth/verify-email?token={token_from_email}

→ 200 OK
{
  "message": "Email verified successfully",
  "email_verified": true,
  "verified_at": "2024-01-15T10:30:00Z",
  "token": "eyJ0eXAiOiJKV1Q...",  // Optional JWT
  "user": { "id": "...", "email": "...", ... }
}
```

### Resend Verification
```http
POST /api/auth/resend-verification
Content-Type: application/json

{
  "email": "user@example.com"
}

→ 200 OK (or 429 if rate limited)
{
  "message": "Verification email sent successfully",
  "email": "user@example.com",
  "sent_at": "2024-01-15T10:30:00Z",
  "expires_in_hours": 24
}
```

### Login (Blocks Unverified Users)
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "user123",
  "password": "SecurePass123!"
}

→ 403 Forbidden (if unverified and EMAIL_VERIFICATION_REQUIRED=true)
{
  "error": {
    "type": "forbidden",
    "message": "Email not verified. Please check your email...",
    "status": 403
  }
}

→ 200 OK (if verified)
{
  "access_token": "eyJ0eXAiOiJKV1Q...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": { ... }
}
```

---

## 🔐 Security Features

| Feature | Implementation |
|---------|----------------|
| **Token Generation** | 32 bytes cryptographic random (256-bit entropy) |
| **Token Encoding** | Base58 (URL-safe, no special chars) |
| **Token Storage** | SHA-256 hashed in database |
| **Token Expiry** | 24 hours from sent_at |
| **Token Usage** | One-time use (cleared after verification) |
| **Rate Limiting** | 5-minute cooldown on resends |
| **Email Validation** | Format check, max length 128 chars |

---

## 📊 Database Schema Changes

```sql
-- Added to users table:
email_verified BOOLEAN NOT NULL DEFAULT false,
email_verification_token VARCHAR(128),           -- SHA-256 hash
email_verification_sent_at TIMESTAMPTZ,
email_verification_expires_at TIMESTAMPTZ,       -- sent_at + 24 hours
email_verified_at TIMESTAMPTZ,

-- Indexes for performance:
CREATE INDEX idx_users_email_verified ON users(email_verified);
CREATE INDEX idx_users_verification_token ON users(email_verification_token);
CREATE INDEX idx_users_verification_expires ON users(email_verification_expires_at);
```

---

## 🧪 Testing

### Run Tests
```bash
# Email verification tests (13 tests)
cargo test --test email_verification_test

# Email service tests (2 tests)
cargo test email_service

# Email templates tests (4 tests)
cargo test email_templates

# Token service tests (6 tests)
cargo test token_service

# All tests
cargo test
```

### Manual Testing Flow
```bash
# 1. Register
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Pass123!","username":"test"}'

# 2. Get token from email (or database in dev)
# SELECT email_verification_token FROM users WHERE email = 'test@example.com';

# 3. Verify
curl "http://localhost:8080/api/auth/verify-email?token=YOUR_TOKEN_HERE"

# 4. Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"Pass123!"}'
```

---

## ⚙️ Configuration Options

### Development Mode (Skip Verification)
```env
EMAIL_ENABLED=false                      # Don't actually send emails
EMAIL_VERIFICATION_REQUIRED=false        # Allow login without verification
```

### Production Mode (Enforce Verification)
```env
EMAIL_ENABLED=true                       # Send emails via SMTP
EMAIL_VERIFICATION_REQUIRED=true         # Block unverified users
AUTO_LOGIN_AFTER_VERIFICATION=true       # Return JWT after verification
```

### SMTP Providers

#### Gmail
```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password          # Generate at myaccount.google.com/apppasswords
```

#### SendGrid
```env
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
```

#### AWS SES
```env
SMTP_HOST=email-smtp.us-east-1.amazonaws.com
SMTP_PORT=587
SMTP_USERNAME=your-ses-access-key-id
SMTP_PASSWORD=your-ses-secret-access-key
```

---

## 🐛 Troubleshooting

### Email Not Sending
```bash
# Check email is enabled
echo $EMAIL_ENABLED  # Should be "true"

# Check SMTP credentials
echo $SMTP_USERNAME
echo $SMTP_HOST

# Run email service test
cargo test email_service::tests::test_send_test_email -- --nocapture
```

### Token Invalid/Expired
- Tokens expire after 24 hours
- Tokens are one-time use (cleared after verification)
- Use resend endpoint to get a new token

### Rate Limited
- Wait 5 minutes between resend requests
- Check `email_verification_sent_at` in database

### Login Still Blocked
```sql
-- Check verification status in database
SELECT email, email_verified, email_verified_at 
FROM users 
WHERE email = 'user@example.com';

-- Manually verify (dev only)
UPDATE users 
SET email_verified = true, email_verified_at = NOW() 
WHERE email = 'user@example.com';
```

---

## 📁 File Locations

| Component | File Path | Lines |
|-----------|-----------|-------|
| Migration | `migrations/20241102000002_add_email_verification.sql` | 100 |
| Email Service | `src/services/email_service.rs` | 240 |
| Email Templates | `src/services/email_templates.rs` | 370 |
| Token Service | `src/services/token_service.rs` | 160 |
| Verification Handlers | `src/handlers/email_verification.rs` | 380 |
| Tests | `tests/email_verification_test.rs` | 550+ |

---

## ✅ Status

| Phase | Status | Tests |
|-------|--------|-------|
| 1. Database Schema | ✅ Complete | Migration applied |
| 2. Email Service | ✅ Complete | 2/2 passing |
| 3. Email Templates | ✅ Complete | 4/4 passing |
| 4. Token Service | ✅ Complete | 6/6 passing |
| 5. API Endpoints | ✅ Complete | 0 errors |
| 6. Registration/Login | ✅ Complete | 0 errors |
| 7. Testing | ✅ Complete | 13/13 passing |

**Total**: 1,800+ lines of code, 25/25 tests passing, 0 compilation errors

---

## 🚀 Next Steps

1. **Frontend Integration** - Create `/verify` page to handle verification links
2. **Admin Tools** - Manual verification, view unverified users
3. **Monitoring** - Track email delivery rates, failed sends
4. **Production** - Configure SPF/DKIM, use SendGrid/AWS SES

---

## 💡 Quick Tips

- **Development**: Set `EMAIL_ENABLED=false` to skip sending emails
- **Testing**: Use `cargo test --test email_verification_test` for fast feedback
- **Debugging**: Check `email_verification_sent_at` and `email_verification_expires_at` in database
- **Security**: Never expose plain tokens in logs or database
- **Performance**: Indexes ensure fast token lookups even with millions of users

---

For detailed documentation, see `EMAIL_VERIFICATION_SUMMARY.md`
