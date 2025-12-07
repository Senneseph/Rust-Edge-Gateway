//! Email (SMTP) Service Connector

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::ServiceConnector;
use crate::api::ServiceType;

/// SMTP encryption mode
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SmtpEncryption {
    /// No encryption
    None,
    /// STARTTLS (upgrade connection to TLS)
    #[default]
    Starttls,
    /// Implicit TLS (TLS from start)
    Tls,
}

/// Email (SMTP) connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP host
    pub host: String,
    /// SMTP port
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    /// Username for authentication
    pub username: Option<String>,
    /// Password (not serialized in responses)
    #[serde(skip_serializing, default)]
    pub password: Option<String>,
    /// Encryption mode
    #[serde(default)]
    pub encryption: SmtpEncryption,
    /// Default "From" address
    pub from_address: String,
    /// Default "From" name
    pub from_name: Option<String>,
    /// Reply-To address
    pub reply_to: Option<String>,
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    /// Maximum retries for sending
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

fn default_smtp_port() -> u16 { 587 }
fn default_timeout() -> u32 { 30 }
fn default_retries() -> u32 { 3 }

/// Email service connector
pub struct EmailConnector {
    config: EmailConfig,
}

impl EmailConnector {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }
    
    /// Get the effective port based on encryption
    pub fn effective_port(&self) -> u16 {
        if self.config.port != 587 {
            return self.config.port;
        }
        // Default ports by encryption
        match self.config.encryption {
            SmtpEncryption::None => 25,
            SmtpEncryption::Starttls => 587,
            SmtpEncryption::Tls => 465,
        }
    }
    
    /// Get the connection URL (informational)
    pub fn connection_url(&self) -> String {
        let protocol = match self.config.encryption {
            SmtpEncryption::None => "smtp",
            SmtpEncryption::Starttls => "smtp+starttls",
            SmtpEncryption::Tls => "smtps",
        };
        format!(
            "{}://{}:{}",
            protocol,
            self.config.host,
            self.effective_port()
        )
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &EmailConfig {
        &self.config
    }
}

impl ServiceConnector for EmailConnector {
    fn service_type(&self) -> ServiceType {
        ServiceType::Email
    }
    
    fn test_connection(&self) -> Result<()> {
        // Placeholder - actual implementation would use lettre crate
        // Would attempt to connect and verify SMTP handshake
        Ok(())
    }
    
    fn connection_info(&self) -> Value {
        json!({
            "type": "email",
            "host": self.config.host,
            "port": self.effective_port(),
            "encryption": format!("{:?}", self.config.encryption).to_lowercase(),
            "from_address": self.config.from_address,
            "from_name": self.config.from_name,
            "reply_to": self.config.reply_to,
            "timeout_seconds": self.config.timeout_seconds,
            "requires_auth": self.config.username.is_some(),
        })
    }
}

