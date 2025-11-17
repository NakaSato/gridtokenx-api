# Email Verification Resend - Improved Response Logic

## Overview

This document describes the improvements made to the email verification resend endpoint (`POST /api/auth/resend-verification`) to provide better user experience by distinguishing between three distinct scenarios:

1. **Already Verified** - User's email is already verified
2. **Expired Token** - User's verification token has expired
3. **Rate Limited** - User is making requests too frequently

## Problem Statement

### Before Improvements

The previous implementation had the following issues:

1. **Already verified users received rate limit errors** instead of a success message
2. **No distinction between expired tokens and valid tokens** - both were subject to the same rate limiting
3. **Unhelpful error messages** - rate limit errors didn't indicate how long to wait

### User Experience Issues

- Users who already verified their email but tried to resend got confusing error messages
- Users with expired tokens had to wait 30 seconds even though their token was already invalid
- Rate limit errors didn't tell users how long to wait before retrying

## Solution

### New Response Logic

The improved implementation uses a three-stage validation approach:

```
1. Check if already verified → Return 200 with "already_verified" status
   ↓
2. Check if token is expired → Generate new token, send email, return 200 with "expired_resent" status
   ↓
3. Check rate limiting (only for valid tokens) → Return 429 with retry time
   ↓
4. Generate new token and send email → Return 200 with "sent" status
```

### Response Types

#### 1. Already Verified (HTTP 200)

**When:** User's `email_verified = true` in database

**Response:**
```json
{
  "message": "Email is already verified. No action needed.",
  "email": "user@example.com",
  "sent_at": "2025-11-17T10:30:00Z",
  "expires_in_hours": 0,
  "status": "already_verified"
}
```

**Key Points:**
- Returns success (200 OK) instead of error
- `expires_in_hours` is 0 (no new token generated)
- `status` field indicates "already_verified"
- No email is sent

#### 2. Expired Token - New Email Sent (HTTP 200)

**When:** User's `email_verification_expires_at < NOW()`

**Response:**
```json
{
  "message": "Your verification token has expired. A new verification email has been sent! Please check your inbox.",
  "email": "user@example.com",
  "sent_at": "2025-11-17T10:30:00Z",
  "expires_in_hours": 24,
  "status": "expired_resent"
}
```

**Key Points:**
- Returns success (200 OK)
- **Bypasses rate limiting** - expired tokens can be resent immediately
- New token generated and email sent
- `status` field indicates "expired_resent"
- Message clearly indicates token was expired

#### 3. Rate Limited (HTTP 429)

**When:** User makes requests within 30 seconds of previous request (for non-expired tokens)

**Response:**
```json
{
  "error": {
    "code": "RATE_9001",
    "code_number": 9001,
    "message": "Rate limit exceeded. Please wait 28 seconds before retrying",
    "retry_after": 28
  },
  "request_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "timestamp": "2025-11-17T10:30:00Z"
}
```

**Key Points:**
- Returns 429 Too Many Requests
- `retry_after` field shows exact seconds to wait
- `Retry-After` HTTP header included
- Message includes wait time in seconds
- Only applies to valid (non-expired) tokens

#### 4. Fresh Resend Request (HTTP 200)

**When:** User's token is valid and rate limit has passed

**Response:**
```json
{
  "message": "Verification email sent successfully! Please check your inbox.",
  "email": "user@example.com",
  "sent_at": "2025-11-17T10:30:00Z",
  "expires_in_hours": 24,
  "status": "sent"
}
```

**Key Points:**
- Returns success (200 OK)
- New token generated (old token invalidated)
- Email sent
- `status` field indicates "sent"

## Implementation Details

### Code Changes

#### 1. New Response Field: `status`

Added optional `status` field to `ResendVerificationResponse`:

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct ResendVerificationResponse {
    pub message: String,
    pub email: String,
    pub sent_at: String,
    pub expires_in_hours: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,  // NEW FIELD
}
```

**Possible values:**
- `"already_verified"` - Email already verified
- `"expired_resent"` - Token was expired, new one sent
- `"sent"` - Fresh resend request

#### 2. New Error Variant: `RateLimitWithRetry`

Added new error type with wait time:

```rust
#[error("Rate limit exceeded. Please wait {0} seconds before retrying")]
RateLimitWithRetry(i64),
```

**Benefits:**
- Includes exact wait time in error message
- Sets `retry_after` field in JSON response
- Sets `Retry-After` HTTP header

#### 3. Restructured Validation Logic

```rust
// 1. Check if already verified FIRST
if user.email_verified {
    return Ok(Json(ResendVerificationResponse {
        status: Some("already_verified".to_string()),
        expires_in_hours: 0,
        // ...
    }));
}

// 2. Check if token is expired
let is_token_expired = if let Some(expires_at) = user.email_verification_expires_at {
    expires_at < Utc::now()
} else {
    true  // No token = treat as expired
};

// 3. Rate limiting - BUT skip if token is expired
if !is_token_expired {
    if let Some(expires_at) = user.email_verification_expires_at {
        let sent_at = expires_at - Duration::hours(verification_expiry_hours);
        let time_since_sent = Utc::now() - sent_at;
        
        if time_since_sent < Duration::seconds(30) {
            let wait_seconds = 30 - time_since_sent.num_seconds();
            return Err(ApiError::RateLimitWithRetry(wait_seconds));
        }
    }
}

// 4. Generate token and send email
// ...

// 5. Return appropriate status
let (message, status) = if is_token_expired {
    ("Your verification token has expired. A new verification email has been sent!", 
     Some("expired_resent".to_string()))
} else {
    ("Verification email sent successfully!", 
     Some("sent".to_string()))
};
```

### Files Modified

1. **`api-gateway/src/handlers/email_verification.rs`**
   - Added `status` field to `ResendVerificationResponse`
   - Restructured `resend_verification()` handler logic
   - Added expired token detection
   - Improved response messages

2. **`api-gateway/src/error.rs`**
   - Added `RateLimitWithRetry(i64)` error variant
   - Updated error handling to include wait time in response
   - Added `Retry-After` header for rate limit errors

3. **`tests/integration/email-verification-resend.test.ts`** (NEW)
   - Comprehensive test suite for all scenarios
   - Tests for already verified users
   - Tests for expired tokens
   - Tests for rate limiting
   - Response structure validation

## Benefits

### 1. Better User Experience

- **Clear feedback** - Users know exactly what happened
- **No confusion** - Already verified users get success messages, not errors
- **Helpful errors** - Rate limit errors show exact wait time

### 2. Improved Security

- **Expired tokens bypass rate limiting** - Prevents users from being locked out after 24 hours
- **Rate limiting still works** - Prevents spam for valid tokens
- **One-time use tokens** - Tokens are invalidated after successful verification

### 3. API Consistency

- **Structured responses** - All success cases return 200 with consistent structure
- **Status field** - Clients can programmatically handle different scenarios
- **HTTP standards** - Uses proper status codes (200 for success, 429 for rate limiting)

## Testing

### Manual Testing

1. **Test Already Verified User:**
   ```bash
   curl -X POST http://localhost:8080/api/auth/resend-verification \
     -H "Content-Type: application/json" \
     -d '{"email": "verified@example.com"}'
   ```
   **Expected:** HTTP 200 with `status: "already_verified"`

2. **Test Expired Token:**
   ```bash
   # First, manually expire a token in database:
   # UPDATE users SET email_verification_expires_at = NOW() - INTERVAL '1 hour' 
   #   WHERE email = 'expired@example.com';
   
   curl -X POST http://localhost:8080/api/auth/resend-verification \
     -H "Content-Type: application/json" \
     -d '{"email": "expired@example.com"}'
   ```
   **Expected:** HTTP 200 with `status: "expired_resent"`

3. **Test Rate Limiting:**
   ```bash
   # Send first request
   curl -X POST http://localhost:8080/api/auth/resend-verification \
     -H "Content-Type: application/json" \
     -d '{"email": "unverified@example.com"}'
   
   # Immediately send second request
   curl -X POST http://localhost:8080/api/auth/resend-verification \
     -H "Content-Type: application/json" \
     -d '{"email": "unverified@example.com"}'
   ```
   **Expected:** First request returns 200, second returns 429 with `retry_after`

### Automated Testing

Run the integration test suite:

```bash
cd tests
pnpm test:integration email-verification-resend
```

## Migration Notes

### Database Schema

No database migrations required - uses existing columns:
- `email_verified` (BOOLEAN)
- `email_verification_expires_at` (TIMESTAMPTZ)
- `email_verification_sent_at` (TIMESTAMPTZ)

### API Compatibility

**Breaking Changes:** None

**New Fields:**
- `status` field in response (optional, backwards compatible)
- `retry_after` field in error response (for rate limiting)

**Behavior Changes:**
- Already verified users now get 200 instead of 400
- Expired tokens now bypass rate limiting
- Rate limit errors now include exact wait time

### Client Integration

Frontend clients should update to handle the `status` field:

```typescript
const response = await fetch('/api/auth/resend-verification', {
  method: 'POST',
  body: JSON.stringify({ email }),
});

const data = await response.json();

if (response.ok) {
  switch (data.status) {
    case 'already_verified':
      showMessage('Your email is already verified!');
      break;
    case 'expired_resent':
      showMessage('Token expired. A new verification email has been sent!');
      break;
    case 'sent':
      showMessage('Verification email sent. Please check your inbox.');
      break;
  }
} else if (response.status === 429) {
  const waitTime = data.error.retry_after;
  showError(`Please wait ${waitTime} seconds before trying again.`);
}
```

## Configuration

No new configuration variables required. Uses existing settings:

```bash
# From .env file
EMAIL_VERIFICATION_EXPIRY_HOURS=24  # Token expiry time
# Rate limit is hardcoded to 30 seconds (can be made configurable if needed)
```

## Future Enhancements

Possible improvements for future iterations:

1. **Configurable rate limit cooldown** - Make 30-second window configurable
2. **Different cooldowns for different scenarios** - Longer cooldown for expired tokens
3. **User-specific rate limits** - Different limits based on user role or reputation
4. **Audit logging** - Track resend attempts for analytics
5. **Email quota management** - Prevent abuse by limiting daily email sends per user

## References

- **Implementation PR:** [Link to PR]
- **Issue Tracking:** [Link to issue]
- **API Documentation:** `POST /api/auth/resend-verification` endpoint
- **Related Docs:**
  - Email Verification Flow: `docs/technical/EMAIL_VERIFICATION.md`
  - Error Handling: `api-gateway/src/error.rs`
  - Rate Limiting Strategy: `docs/technical/RATE_LIMITING.md`

---

**Last Updated:** November 17, 2025  
**Author:** GridTokenX Development Team  
**Version:** 1.0
