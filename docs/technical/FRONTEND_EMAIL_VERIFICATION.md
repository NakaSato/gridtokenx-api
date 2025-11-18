# Frontend Email Verification Flow

## Overview
This document explains how the frontend handles email verification and automatic wallet generation in the GridTokenX platform.

## Complete User Registration Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   User      │────▶│   Frontend   │────▶│   Backend   │────▶│   Database   │
│  Registration│     │   Register   │     │    API      │     │              │
└─────────────┘     └──────────────┘     └─────────────┘     └──────────────┘
                            │                     │
                            │                     │
                            ▼                     ▼
                    ┌──────────────┐     ┌─────────────┐
                    │ Show Success │     │ Send Email  │
                    │   Message    │     │ with Token  │
                    └──────────────┘     └─────────────┘
                                                │
                                                ▼
                                        ┌──────────────────┐
                                        │ User clicks link │
                                        │ in email         │
                                        └──────────────────┘
                                                │
                                                ▼
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Backend   │◀────│   Frontend   │◀────│ URL: /verify│
│ Auto-creates│     │ Extract token│     │ -email?     │
│ Solana Wallet     │ & call API   │     │ token=xxx   │
└─────────────┘     └──────────────┘     └─────────────┘
```

## Step-by-Step Implementation

### 1. Registration Page (No Wallet Required!)

**Frontend Form:**
```typescript
// src/pages/Register.tsx
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

interface RegisterFormData {
  username: string;
  email: string;
  password: string;
  first_name: string;
  last_name: string;
  role: 'user' | 'producer' | 'consumer';
  // ❌ NO wallet_address field!
}

export function RegisterPage() {
  const [formData, setFormData] = useState<RegisterFormData>({
    username: '',
    email: '',
    password: '',
    first_name: '',
    last_name: '',
    role: 'user'
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      const response = await fetch('http://localhost:8080/api/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(formData),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Registration failed');
      }

      const data = await response.json();
      
      // Show success message and redirect
      navigate('/registration-success', { 
        state: { email: formData.email } 
      });
      
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="register-page">
      <h1>Create Your Account</h1>
      <p>Join GridTokenX - No wallet needed yet!</p>
      
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          placeholder="Username"
          value={formData.username}
          onChange={(e) => setFormData({...formData, username: e.target.value})}
          required
        />
        
        <input
          type="email"
          placeholder="Email"
          value={formData.email}
          onChange={(e) => setFormData({...formData, email: e.target.value})}
          required
        />
        
        <input
          type="password"
          placeholder="Password (min 8 characters)"
          value={formData.password}
          onChange={(e) => setFormData({...formData, password: e.target.value})}
          required
          minLength={8}
        />
        
        <input
          type="text"
          placeholder="First Name"
          value={formData.first_name}
          onChange={(e) => setFormData({...formData, first_name: e.target.value})}
          required
        />
        
        <input
          type="text"
          placeholder="Last Name"
          value={formData.last_name}
          onChange={(e) => setFormData({...formData, last_name: e.target.value})}
          required
        />
        
        {error && <div className="error">{error}</div>}
        
        <button type="submit" disabled={loading}>
          {loading ? 'Creating Account...' : 'Register'}
        </button>
      </form>
    </div>
  );
}
```

### 2. Registration Success Page

```typescript
// src/pages/RegistrationSuccess.tsx
import { useLocation } from 'react-router-dom';

export function RegistrationSuccessPage() {
  const location = useLocation();
  const email = location.state?.email;

  return (
    <div className="success-page">
      <div className="success-icon">✅</div>
      <h1>Registration Successful!</h1>
      <p>
        We've sent a verification email to <strong>{email}</strong>
      </p>
      <div className="info-box">
        <h3>Next Steps:</h3>
        <ol>
          <li>Check your email inbox (and spam folder)</li>
          <li>Click the verification link in the email</li>
          <li>Your Solana wallet will be automatically created</li>
          <li>You can then login and start trading energy!</li>
        </ol>
      </div>
      <p className="note">
        💡 <strong>Note:</strong> You don't need to create a wallet manually - 
        we'll generate a secure Solana wallet for you when you verify your email!
      </p>
    </div>
  );
}
```

### 3. Email Verification Handler Page

**This is the page that handles the email link click:**

```typescript
// src/pages/VerifyEmail.tsx
import { useEffect, useState } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';

interface VerificationResponse {
  message: string;
  email_verified: boolean;
  wallet_address: string;
  username?: string;
  email?: string;
}

export function VerifyEmailPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const [status, setStatus] = useState<'verifying' | 'success' | 'error'>('verifying');
  const [result, setResult] = useState<VerificationResponse | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    const verifyEmail = async () => {
      // Extract token from URL query parameter
      const token = searchParams.get('token');
      
      if (!token) {
        setStatus('error');
        setError('Invalid verification link. No token provided.');
        return;
      }

      try {
        // Call the verification API endpoint
        const response = await fetch(
          `http://localhost:8080/api/auth/verify-email?token=${token}`,
          {
            method: 'GET',
            headers: {
              'Content-Type': 'application/json',
            },
          }
        );

        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(
            errorData.error?.message || 
            errorData.message || 
            'Verification failed'
          );
        }

        const data: VerificationResponse = await response.json();
        setResult(data);
        setStatus('success');

        // Redirect to login after 3 seconds
        setTimeout(() => {
          navigate('/login', { 
            state: { 
              message: 'Email verified! Please login to continue.',
              walletCreated: true 
            } 
          });
        }, 3000);

      } catch (err: any) {
        setStatus('error');
        setError(err.message);
      }
    };

    verifyEmail();
  }, [searchParams, navigate]);

  if (status === 'verifying') {
    return (
      <div className="verify-page">
        <div className="spinner">⏳</div>
        <h1>Verifying Your Email...</h1>
        <p>Please wait while we verify your account</p>
      </div>
    );
  }

  if (status === 'success' && result) {
    return (
      <div className="verify-page success">
        <div className="success-icon">🎉</div>
        <h1>Email Verified Successfully!</h1>
        
        <div className="wallet-created">
          <h2>🔐 Your Solana Wallet is Ready!</h2>
          <p>We've automatically created a secure Solana wallet for you:</p>
          <div className="wallet-address">
            <code>{result.wallet_address}</code>
          </div>
          <button 
            onClick={() => navigator.clipboard.writeText(result.wallet_address)}
            className="copy-button"
          >
            📋 Copy Wallet Address
          </button>
        </div>

        <div className="next-steps">
          <h3>What's Next?</h3>
          <ul>
            <li>✅ Your email is verified</li>
            <li>✅ Your Solana wallet is created</li>
            <li>✅ You can now login to the platform</li>
            <li>🚀 Start trading energy tokens!</li>
          </ul>
        </div>

        <p className="redirect-notice">
          Redirecting to login page in 3 seconds...
        </p>
        
        <button onClick={() => navigate('/login')}>
          Login Now →
        </button>
      </div>
    );
  }

  return (
    <div className="verify-page error">
      <div className="error-icon">❌</div>
      <h1>Verification Failed</h1>
      <p className="error-message">{error}</p>
      <div className="help-box">
        <h3>Possible reasons:</h3>
        <ul>
          <li>The verification link has expired (24 hours)</li>
          <li>The link has already been used</li>
          <li>Invalid or malformed token</li>
        </ul>
      </div>
      <button onClick={() => navigate('/resend-verification')}>
        Request New Verification Email
      </button>
    </div>
  );
}
```

### 4. Router Configuration

```typescript
// src/App.tsx or src/router.tsx
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { RegisterPage } from './pages/Register';
import { RegistrationSuccessPage } from './pages/RegistrationSuccess';
import { VerifyEmailPage } from './pages/VerifyEmail';
import { LoginPage } from './pages/Login';

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/registration-success" element={<RegistrationSuccessPage />} />
        <Route path="/verify-email" element={<VerifyEmailPage />} />
        <Route path="/login" element={<LoginPage />} />
        {/* other routes */}
      </Routes>
    </BrowserRouter>
  );
}
```

## Email Example

**What users receive in their inbox:**

```
Subject: Verify Your Email - GridTokenX Platform

Hello john_doe,

Thank you for registering with GridTokenX Platform. We're excited to 
have you join our peer-to-peer energy trading network!

To complete your registration and start trading energy tokens, please 
verify your email address by clicking the button below:

┌──────────────────────────┐
│  Verify Email Address    │  ← Links to: http://localhost:3000/verify-email?token=U41xd8iA...IjaQ
└──────────────────────────┘

If the button doesn't work, copy and paste this link:
http://localhost:3000/verify-email?token=U41xd8iA0XhbaCzS3DzUiQTBjB0IjaQ

🔐 Security Note: Your Solana wallet will be automatically created when 
you verify your email. You don't need to do anything else!

This link expires in 24 hours.
```

## API Endpoints Used

### 1. Registration API
```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Doe",
  "role": "user"
}

Response (HTTP 201):
{
  "message": "Registration successful! Please check your email to verify your account.",
  "user": {
    "username": "john_doe",
    "email": "john@example.com",
    "role": "user"
  },
  "email_verification_sent": true,
  "verification_required": false
}
```

### 2. Email Verification API (Auto-generates Wallet!)
```http
GET /api/auth/verify-email?token=U41xd8iA0XhbaCzS3DzUiQTBjB0IjaQ

Response (HTTP 200):
{
  "message": "Email verified successfully! Your Solana wallet has been created.",
  "email_verified": true,
  "wallet_address": "92HhyT6GEdt5c7En8TsxxNMK2VR5gwM7G7HL4jDNg1ZA"
}
```

### 3. Login API (After Verification)
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "john_doe",
  "password": "SecurePass123!"
}

Response (HTTP 200):
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "john_doe",
    "email": "john@example.com",
    "wallet_address": "92HhyT6GEdt5c7En8TsxxNMK2VR5gwM7G7HL4jDNg1ZA",
    "email_verified": true
  }
}
```

## Important Notes

### ✅ What Frontend Should Do:
1. **DO NOT** include `wallet_address` field in registration form
2. **DO** extract `token` from URL query params in verify-email page
3. **DO** call `GET /api/auth/verify-email?token=xxx` to trigger wallet creation
4. **DO** display the generated wallet address to the user after verification
5. **DO** redirect to login after successful verification

### ❌ What Frontend Should NOT Do:
1. **DO NOT** create wallets on the frontend
2. **DO NOT** ask users for wallet addresses during registration
3. **DO NOT** try to verify email without the token from email link
4. **DO NOT** store the verification token (it's single-use)

## Security Benefits

1. **Automatic Wallet Creation**: Users can't provide malicious wallet addresses
2. **Email Ownership Proof**: Wallet is only created after email verification
3. **Single-Use Tokens**: Each verification link works only once
4. **Time-Limited**: Tokens expire after 24 hours
5. **Backend-Generated**: Wallets are created securely on Solana using proper keypair generation

## Testing the Flow

Use the test script to verify everything works:

```bash
cd api-gateway/scripts
./test_complete_flow.sh
```

This validates:
- Registration without wallet_address (✓)
- Registration rejects wallet_address field (✓)
- Email verification generates wallet (✓)
- Login works after verification (✓)
- Profile endpoint returns wallet address (✓)

## Environment Configuration

**Backend (.env):**
```bash
# Email service configuration
EMAIL_VERIFICATION_BASE_URL=http://localhost:3000  # Frontend URL
EMAIL_HOST=localhost
EMAIL_PORT=1025
EMAIL_USERNAME=
EMAIL_PASSWORD=
EMAIL_FROM=noreply@gridtokenx.com

# Verification token expiry (hours)
EMAIL_VERIFICATION_EXPIRY_HOURS=24
```

**Frontend (.env):**
```bash
# API base URL
VITE_API_BASE_URL=http://localhost:8080
```

## Troubleshooting

### "Invalid or expired verification token"
- Token has been used already
- Token expired (>24 hours old)
- Token was malformed in URL

**Solution:** Use resend verification endpoint:
```typescript
fetch('http://localhost:8080/api/auth/resend-verification', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ email: 'user@example.com' })
});
```

### "Email already verified"
- User clicked the link twice
- Email was already verified previously

**Solution:** Just redirect to login - user can login now!

### "Wallet address format is invalid"
- This shouldn't happen if using backend-generated wallets
- Backend creates proper Base58-encoded Solana public keys

**Solution:** Contact support - this is a backend issue.
