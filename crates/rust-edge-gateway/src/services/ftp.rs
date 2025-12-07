//! FTP/SFTP Service Connector

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::ServiceConnector;
use crate::api::ServiceType;

/// FTP connection protocol
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FtpProtocol {
    /// Standard FTP (unencrypted)
    #[default]
    Ftp,
    /// FTP over TLS (explicit)
    Ftps,
    /// SSH File Transfer Protocol
    Sftp,
}

/// FTP connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    /// FTP host
    pub host: String,
    /// FTP port
    #[serde(default = "default_ftp_port")]
    pub port: u16,
    /// Username
    pub username: String,
    /// Password (not serialized in responses)
    #[serde(skip_serializing, default)]
    pub password: Option<String>,
    /// SSH private key path (for SFTP)
    #[serde(skip_serializing)]
    pub private_key_path: Option<String>,
    /// Protocol to use
    #[serde(default)]
    pub protocol: FtpProtocol,
    /// Base directory on the server
    #[serde(default)]
    pub base_path: Option<String>,
    /// Passive mode (for FTP/FTPS)
    #[serde(default = "default_true")]
    pub passive_mode: bool,
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

fn default_ftp_port() -> u16 { 21 }
fn default_timeout() -> u32 { 30 }
fn default_true() -> bool { true }

/// FTP service connector
pub struct FtpConnector {
    config: FtpConfig,
}

impl FtpConnector {
    pub fn new(config: FtpConfig) -> Self {
        Self { config }
    }
    
    /// Get the effective port based on protocol
    pub fn effective_port(&self) -> u16 {
        if self.config.port != 21 {
            return self.config.port;
        }
        // Default ports by protocol
        match self.config.protocol {
            FtpProtocol::Ftp | FtpProtocol::Ftps => 21,
            FtpProtocol::Sftp => 22,
        }
    }
    
    /// Get the connection URL (informational)
    pub fn connection_url(&self) -> String {
        let protocol = match self.config.protocol {
            FtpProtocol::Ftp => "ftp",
            FtpProtocol::Ftps => "ftps",
            FtpProtocol::Sftp => "sftp",
        };
        format!(
            "{}://{}@{}:{}{}",
            protocol,
            self.config.username,
            self.config.host,
            self.effective_port(),
            self.config.base_path.as_deref().unwrap_or("")
        )
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &FtpConfig {
        &self.config
    }
}

impl ServiceConnector for FtpConnector {
    fn service_type(&self) -> ServiceType {
        ServiceType::Ftp
    }
    
    fn test_connection(&self) -> Result<()> {
        // Placeholder - actual implementation would use ftp/ssh2 crates
        // Would attempt to connect and list directory
        Ok(())
    }
    
    fn connection_info(&self) -> Value {
        json!({
            "type": "ftp",
            "protocol": format!("{:?}", self.config.protocol).to_lowercase(),
            "host": self.config.host,
            "port": self.effective_port(),
            "username": self.config.username,
            "base_path": self.config.base_path,
            "passive_mode": self.config.passive_mode,
            "timeout_seconds": self.config.timeout_seconds,
        })
    }
}

