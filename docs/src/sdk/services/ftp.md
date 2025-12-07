# FTP/SFTP Service

Transfer files to and from remote servers using FTP, FTPS, or SFTP.

## Configuration

### FTP (Unencrypted)

```json
{
  "service_type": "ftp",
  "config": {
    "host": "ftp.example.com",
    "port": 21,
    "username": "user",
    "password": "secret",
    "protocol": "ftp",
    "base_path": "/uploads",
    "passive_mode": true,
    "timeout_seconds": 30
  }
}
```

### FTPS (FTP over TLS)

```json
{
  "service_type": "ftp",
  "config": {
    "host": "ftp.example.com",
    "port": 21,
    "username": "user",
    "password": "secret",
    "protocol": "ftps",
    "passive_mode": true
  }
}
```

### SFTP (SSH File Transfer)

```json
{
  "service_type": "ftp",
  "config": {
    "host": "sftp.example.com",
    "port": 22,
    "username": "user",
    "password": "secret",
    "protocol": "sftp",
    "base_path": "/home/user/uploads"
  }
}
```

Or with SSH key authentication:

```json
{
  "service_type": "ftp",
  "config": {
    "host": "sftp.example.com",
    "port": 22,
    "username": "user",
    "private_key_path": "/path/to/id_rsa",
    "protocol": "sftp"
  }
}
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | required | FTP server hostname |
| `port` | u16 | 21/22 | Server port (21 for FTP/FTPS, 22 for SFTP) |
| `username` | string | required | Login username |
| `password` | string | null | Login password |
| `private_key_path` | string | null | Path to SSH private key (SFTP only) |
| `protocol` | string | "ftp" | Protocol: "ftp", "ftps", or "sftp" |
| `base_path` | string | null | Default directory on server |
| `passive_mode` | bool | true | Use passive mode (FTP/FTPS only) |
| `timeout_seconds` | u32 | 30 | Connection timeout |

## Usage

### Upload a File

```rust
use rust_edge_gateway_sdk::prelude::*;

fn handle(req: Request) -> Response {
    let ftp = FtpPool { pool_id: "uploads".to_string() };
    
    // Upload content
    let content = req.body.as_ref().unwrap();
    let result = ftp.put("/reports/daily.csv", content.as_bytes());
    
    match result {
        Ok(()) => Response::ok(json!({"uploaded": true})),
        Err(e) => Response::internal_error(e.to_string()),
    }
}
```

### Download a File

```rust
fn handle(req: Request) -> Response {
    let ftp = FtpPool { pool_id: "files".to_string() };
    
    let filename = req.path_param("filename").unwrap();
    let path = format!("/data/{}", filename);
    
    match ftp.get(&path) {
        Ok(content) => Response::new(200)
            .with_header("Content-Type", "application/octet-stream")
            .with_body(content),
        Err(e) => Response::not_found(),
    }
}
```

### List Directory

```rust
fn handle(req: Request) -> Response {
    let ftp = FtpPool { pool_id: "files".to_string() };
    
    match ftp.list("/reports") {
        Ok(files) => Response::ok(json!({"files": files})),
        Err(e) => Response::internal_error(e.to_string()),
    }
}
```

## Use Cases

- **File uploads** - Accept user uploads and store on FTP server
- **Report distribution** - Upload generated reports to partner SFTP servers
- **Data import** - Download files from vendor FTP for processing
- **Backup** - Archive data to remote storage
- **Legacy integration** - Connect to systems that only support FTP

## Security Notes

1. **Prefer SFTP** - Uses SSH encryption, most secure option
2. **Use FTPS if SFTP unavailable** - TLS encryption for FTP
3. **Avoid plain FTP** - Credentials sent in cleartext
4. **Use SSH keys** - More secure than passwords for SFTP
5. **Restrict base_path** - Limit access to specific directories

