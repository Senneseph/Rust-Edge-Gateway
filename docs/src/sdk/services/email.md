# Email (SMTP) Service

Send emails from your handlers using SMTP.

## Configuration

### Basic SMTP with STARTTLS

```json
{
  "service_type": "email",
  "config": {
    "host": "smtp.example.com",
    "port": 587,
    "username": "sender@example.com",
    "password": "app-password",
    "encryption": "starttls",
    "from_address": "noreply@example.com",
    "from_name": "My App",
    "reply_to": "support@example.com"
  }
}
```

### Gmail SMTP

```json
{
  "service_type": "email",
  "config": {
    "host": "smtp.gmail.com",
    "port": 587,
    "username": "your-email@gmail.com",
    "password": "your-app-password",
    "encryption": "starttls",
    "from_address": "your-email@gmail.com",
    "from_name": "Your Name"
  }
}
```

### Implicit TLS (Port 465)

```json
{
  "service_type": "email",
  "config": {
    "host": "smtp.example.com",
    "port": 465,
    "username": "sender@example.com",
    "password": "secret",
    "encryption": "tls",
    "from_address": "noreply@example.com"
  }
}
```

### Local SMTP (No Auth)

```json
{
  "service_type": "email",
  "config": {
    "host": "localhost",
    "port": 25,
    "encryption": "none",
    "from_address": "app@localhost"
  }
}
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | required | SMTP server hostname |
| `port` | u16 | 587 | SMTP port |
| `username` | string | null | Auth username (usually email address) |
| `password` | string | null | Auth password or app password |
| `encryption` | string | "starttls" | Encryption: "none", "starttls", or "tls" |
| `from_address` | string | required | Default sender email |
| `from_name` | string | null | Default sender display name |
| `reply_to` | string | null | Default reply-to address |
| `timeout_seconds` | u32 | 30 | Connection timeout |
| `max_retries` | u32 | 3 | Send retry attempts |

## Usage

### Send a Simple Email

```rust
use rust_edge_gateway_sdk::prelude::*;

fn handle(req: Request) -> Response {
    let email = EmailPool { pool_id: "notifications".to_string() };
    
    let result = email.send(
        "user@example.com",
        "Welcome!",
        "Thanks for signing up.",
    );
    
    match result {
        Ok(()) => Response::ok(json!({"sent": true})),
        Err(e) => Response::internal_error(e.to_string()),
    }
}
```

### Send HTML Email

```rust
fn handle(req: Request) -> Response {
    let email = EmailPool { pool_id: "notifications".to_string() };
    
    let html = r#"
        <h1>Welcome!</h1>
        <p>Thanks for joining us.</p>
        <a href="https://example.com/verify">Verify your email</a>
    "#;
    
    let result = email.send_html(
        "user@example.com",
        "Welcome to Our App",
        html,
    );
    
    match result {
        Ok(()) => Response::ok(json!({"sent": true})),
        Err(e) => Response::internal_error(e.to_string()),
    }
}
```

### Send with Custom From

```rust
let result = email.send_from(
    "support@example.com",  // From
    "Support Team",         // From name
    "user@example.com",     // To
    "Your Ticket #123",     // Subject
    "We've received your support request...",
);
```

## Common Providers

| Provider | Host | Port | Encryption |
|----------|------|------|------------|
| Gmail | smtp.gmail.com | 587 | starttls |
| Outlook | smtp.office365.com | 587 | starttls |
| SendGrid | smtp.sendgrid.net | 587 | starttls |
| Mailgun | smtp.mailgun.org | 587 | starttls |
| Amazon SES | email-smtp.{region}.amazonaws.com | 587 | starttls |

## Best Practices

1. **Use app passwords** - Don't use your main account password
2. **Set up SPF/DKIM** - Improve deliverability
3. **Handle failures** - Emails can fail; log and retry
4. **Rate limit** - Don't spam; respect provider limits
5. **Use templates** - Consistent formatting
6. **Test with sandbox** - Use test mode before production

