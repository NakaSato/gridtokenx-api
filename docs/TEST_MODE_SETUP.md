# Test Mode Setup Guide

## Overview

The GridTokenX API Gateway supports a **TEST_MODE** environment variable that bypasses email verification requirements during testing. This enables automated integration tests to run without requiring manual email verification.

## How It Works

When `TEST_MODE=true` is set:

1. **Email verification is bypassed** during login
2. **Users can authenticate** immediately after registration
3. **Test workflows** can run end-to-end without manual intervention

## Implementation Details

### Authentication Flow Changes

In `src/handlers/auth.rs`, the login handler checks for test mode:

```rust
// Check email verification requirements
let user = sqlx::query_as!(
    UserRow,
    "SELECT id, email, password_hash, role, email_verified, created_at, updated_at FROM users WHERE email = $1",
    credentials.email
)
.fetch_optional(&state.db)
.await?;

if let Some(user) = user {
    // Check if user needs email verification
    let needs_verification = !user.email_verified && 
        !config.test_mode &&  // <-- Test mode bypass
        user.role != "admin"; // Admins bypass verification
    
    if needs_verification {
        return Err(error::AppError::Unauthorized(
            "Email verification required. Please check your email.".to_string()
        ));
    }
}
```

### Configuration Changes

In `src/config/mod.rs`, test mode is added to the configuration:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub test_mode: bool,
    // ... other fields
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let test_mode = std::env::var("TEST_MODE").is_ok() || 
                       matches!(env::var("ENVIRONMENT").as_deref(), Ok("test"));
        
        Ok(Config {
            test_mode,
            // ... other fields
        })
    }
}
```

## Usage

### Environment Variables

Set either of these environment variables:

```bash
# Method 1: Direct TEST_MODE flag
export TEST_MODE=true

# Method 2: Set environment to "test"
export ENVIRONMENT=test
```

### Running Tests with Test Mode

#### Integration Tests

```bash
cd tests && TEST_MODE=true npm run test:integration -- phase4-energy-tokenization.test.ts
```

#### API Gateway Development

```bash
cd api-gateway && TEST_MODE=true cargo run
```

#### Docker Development

```bash
# In docker-compose.yml or .env file
TEST_MODE=true

# Or when starting services
docker-compose up -d postgres redis
TEST_MODE=true cd api-gateway && cargo run
```

## Database Requirements

Test mode still requires a running database:

```bash
# Start required services
docker-compose up -d postgres redis

# Run tests
TEST_MODE=true npm run test:integration
```

## When to Use Test Mode

### ✅ Use Test Mode For:

- **Automated integration tests**
- **CI/CD pipelines**
- **Development workflows** where email verification is impractical
- **Performance testing** that requires authentication

### ❌ Do NOT Use Test Mode For:

- **Production environments**
- **Staging environments** that should mirror production
- **Security testing** (unless specifically testing bypass mechanisms)
- **User acceptance testing** (UAT should use real email flow)

## Security Considerations

1. **Never enable in production** - Test mode bypasses a critical security control
2. **Use only in isolated environments** - Ensure test databases are separate from production
3. **Monitor for accidental enabling** - Add logging to detect test mode in unexpected environments
4. **Document the bypass** - Ensure team members understand when and why test mode is used

## Troubleshooting

### Tests Still Skipping Email Verification

1. **Check environment variable**:
   ```bash
   echo $TEST_MODE
   # Should output: true
   ```

2. **Verify API Gateway sees the flag**:
   ```bash
   # Check logs for test mode detection
   cd api-gateway && TEST_MODE=true cargo run | grep -i test
   ```

3. **Ensure database is running**:
   ```bash
   docker ps | grep postgres
   ```

### Accidental Production Usage

If test mode is accidentally enabled in production:

1. **Immediately remove the `TEST_MODE=true` environment variable**
2. **Restart the API Gateway service**
3. **Audit user accounts** created during the period
4. **Force email verification** for any unverified accounts

## Related Files

- `src/handlers/auth.rs` - Authentication logic with test mode bypass
- `src/config/mod.rs` - Configuration management
- `tests/README.md` - Test documentation
- `docs/EMAIL_VERIFICATION_PLAN.md` - Email verification documentation

## Examples

### Complete Test Workflow

```bash
# 1. Start required services
docker-compose up -d postgres redis

# 2. Start API Gateway in test mode
cd api-gateway && TEST_MODE=true cargo run &

# 3. Run integration tests
cd tests && TEST_MODE=true npm run test:integration

# 4. Verify results
# All tests should pass without email verification prompts
```

### CI/CD Pipeline Example

```yaml
# .github/workflows/integration-tests.yml
- name: Start Database Services
  run: docker-compose up -d postgres redis

- name: Run Integration Tests
  env:
    TEST_MODE: true
  run: |
    cd tests && npm run test:integration
```

---

**Last Updated**: November 9, 2025  
**Version**: 1.0  
**Maintainer**: GridTokenX Development Team
