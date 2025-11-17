# Authentication Guide - API Gateway

## Overview
The API Gateway provides multiple authentication methods for users to sign in, sign up, and manage their accounts.

## Available Authentication Endpoints

### 1. User Registration (Sign Up)

**Endpoint:** `POST /auth/register`

**Request Body:**
```json
{
  "username": "john_doe",
  "email": "john.doe@example.com",
  "password": "SecurePassword123!",
  "role": "user",
  "first_name": "John",
  "last_name": "Doe",
  "wallet_address": "optional-solana-wallet-address" // Optional
}
```

**Validation Rules:**
- **username**: 3-50 characters
- **email**: Valid email format
- **password**: 8-128 characters (minimum 8 characters for security)
- **role**: Must be one of: `user`, `admin`, `ami`
- **first_name**: 1-100 characters
- **last_name**: 1-100 characters
- **wallet_address**: 32-44 characters (Solana address format) - Optional

**Roles:**
- **user**: Regular users with basic permissions (energy read/write, trading, profile management)
- **admin**: Administrators with full system access
- **ami**: Automated Metering Infrastructure devices (smart meters)

**Success Response (201 Created):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "username": "john_doe",
    "email": "john.doe@example.com",
    "role": "user",
    "blockchain_registered": false
  }
}
```

**Example using curl:**
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john.doe@example.com",
    "password": "SecurePassword123!",
    "role": "user",
    "first_name": "John",
    "last_name": "Doe"
  }'
```

---

### 2. User Login (Sign In)

**Endpoint:** `POST /auth/login`

**Request Body:**
```json
{
  "username": "john_doe",
  "password": "SecurePassword123!"
}
```

**Validation Rules:**
- **username**: 3-50 characters
- **password**: 8-128 characters

**Success Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "username": "john_doe",
    "email": "john.doe@example.com",
    "role": "user",
    "blockchain_registered": true
  }
}
```

**Example using curl:**
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "password": "SecurePassword123!"
  }'
```

**Save the access token for authenticated requests!**

---

### 3. Wallet-Based Registration (Alternative Sign Up)

**Endpoint:** `POST /auth/wallet/register`

Register using a Solana wallet address (for blockchain-first users).

**Request Body:**
```json
{
  "username": "crypto_user",
  "email": "user@university.edu",
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "role": "student",
  "department": "Computer Engineering",
  "first_name": "Crypto",
  "last_name": "User",
  "signature": "optional-wallet-signature"
}
```

---

### 4. Wallet-Based Login (Alternative Sign In)

**Endpoint:** `POST /auth/wallet/login`

Login using your Solana wallet address.

**Request Body:**
```json
{
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "signature": "optional-signature-proof"
}
```

---

## Protected Endpoints (Require Authentication)

All protected endpoints require the `Authorization` header with the JWT token:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 5. Get User Profile

**Endpoint:** `GET /auth/profile`

**Headers:**
```
Authorization: Bearer <your_access_token>
```

**Success Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "blockchain_registered": true
}
```

**Example using curl:**
```bash
curl -X GET http://localhost:8080/auth/profile \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

---

### 6. Update User Profile

**Endpoint:** `POST /auth/profile`

**Headers:**
```
Authorization: Bearer <your_access_token>
```

**Request Body (all fields optional):**
```json
{
  "email": "new.email@example.com",
  "first_name": "NewFirstName",
  "last_name": "NewLastName",
  "wallet_address": "new-wallet-address"
}
```

**Example using curl:**
```bash
curl -X POST http://localhost:8080/auth/profile \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Johnny",
    "last_name": "Smith"
  }'
```

---

### 7. Change Password

**Endpoint:** `POST /auth/password`

**Headers:**
```
Authorization: Bearer <your_access_token>
```

**Request Body:**
```json
{
  "current_password": "OldPassword123!",
  "new_password": "NewSecurePassword456!"
}
```

**Example using curl:**
```bash
curl -X POST http://localhost:8080/auth/password \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "OldPassword123!",
    "new_password": "NewSecurePassword456!"
  }'
```

---

## Username Management

### Username Rules
- **Length**: 3-50 characters
- **Uniqueness**: Username must be unique across all users
- **Format**: Case-sensitive, alphanumeric and underscores recommended
- **Cannot be changed**: Once set during registration, usernames are permanent

### Checking Username Availability
Before registering, you can check if a username is already taken by attempting registration. If the username exists, you'll receive an error:

```json
{
  "error": "Username or email already exists"
}
```

---

## Common Error Responses

### 400 Bad Request
```json
{
  "error": "Validation error: username length must be between 3 and 50"
}
```

### 401 Unauthorized
```json
{
  "error": "Invalid credentials"
}
```

### 404 Not Found
```json
{
  "error": "User not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Database error: connection failed"
}
```

---

## Complete User Flow Example

### 1. Register a new user
```bash
TOKEN=$(curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "jane_smith",
    "email": "jane.smith@example.com",
    "password": "MyPassword123!",
    "role": "user",
    "first_name": "Jane",
    "last_name": "Smith"
  }' | jq -r '.access_token')
```

### 2. Use the token to get profile
```bash
curl -X GET http://localhost:8080/auth/profile \
  -H "Authorization: Bearer $TOKEN"
```

### 3. Update profile
```bash
curl -X POST http://localhost:8080/auth/profile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Janet"
  }'
```

### 4. Login again (get new token)
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "jane_smith",
    "password": "MyPassword123!"
  }'
```

---

## Using with Frontend Applications

### JavaScript/TypeScript Example

```typescript
// Register
async function register(userData) {
  const response = await fetch('http://localhost:8080/auth/register', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(userData),
  });
  
  const data = await response.json();
  
  if (response.ok) {
    // Store token in localStorage or secure storage
    localStorage.setItem('access_token', data.access_token);
    return data;
  } else {
    throw new Error(data.error);
  }
}

// Login
async function login(username, password) {
  const response = await fetch('http://localhost:8080/auth/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ username, password }),
  });
  
  const data = await response.json();
  
  if (response.ok) {
    localStorage.setItem('access_token', data.access_token);
    return data;
  } else {
    throw new Error(data.error);
  }
}

// Get Profile
async function getProfile() {
  const token = localStorage.getItem('access_token');
  
  const response = await fetch('http://localhost:8080/auth/profile', {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });
  
  return await response.json();
}

// Update Profile
async function updateProfile(updates) {
  const token = localStorage.getItem('access_token');
  
  const response = await fetch('http://localhost:8080/auth/profile', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(updates),
  });
  
  return await response.json();
}
```

---

## Security Best Practices

1. **Password Requirements**
   - Minimum 8 characters
   - Use a mix of uppercase, lowercase, numbers, and symbols
   - Don't reuse passwords

2. **Token Storage**
   - Store JWT tokens securely (httpOnly cookies or secure storage)
   - Never expose tokens in URLs
   - Tokens expire after 24 hours

3. **HTTPS in Production**
   - Always use HTTPS in production
   - Never send credentials over HTTP

4. **Rate Limiting**
   - The API has rate limiting enabled
   - Excessive failed login attempts may result in temporary blocking

---

## Testing the API

The server is running on: `http://localhost:8080`

Health check: `http://localhost:8080/health`

You can test using:
- **curl** (command line)
- **Postman** (GUI tool)
- **HTTPie** (user-friendly command line)
- **Your browser's Developer Console** (fetch API)

---

## Need Help?

- Check the server logs for detailed error messages
- Verify the database is running: PostgreSQL on port 5432
- Verify Redis is running: port 6379
- Ensure all environment variables are set correctly in `.env`
