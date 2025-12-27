//! reCAPTCHA v3 verification

/// Verify reCAPTCHA v3 token with Google's API
pub async fn verify_recaptcha_token(
    secret_key: &str,
    token: &str,
    action: &str,
) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let url = "https://www.google.com/recaptcha/api/siteverify";

    let params = [("secret", secret_key), ("response", token)];

    let response = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to verify reCAPTCHA: {}", e))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse reCAPTCHA response: {}", e))?;

    if !json["success"].as_bool().unwrap_or(false) {
        return Err("reCAPTCHA verification failed".to_string());
    }

    // Check if the action matches (optional but recommended)
    if let Some(recaptcha_action) = json["action"].as_str() {
        if recaptcha_action != action {
            return Err(format!(
                "reCAPTCHA action mismatch: expected {}, got {}",
                action, recaptcha_action
            ));
        }
    }

    // Check the score - for login actions, we typically want a higher score
    let score = json["score"].as_f64().unwrap_or(0.0);
    if score < 0.5 {
        return Err(format!("reCAPTCHA score too low: {}", score));
    }

    Ok(true)
}

