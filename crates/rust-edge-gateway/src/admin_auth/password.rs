//! Password validation utilities

/// Password validation requirements
const MIN_PASSWORD_LENGTH: usize = 12;
const REQUIRE_UPPERCASE: bool = true;
const REQUIRE_LOWERCASE: bool = true;
const REQUIRE_DIGIT: bool = true;
const REQUIRE_SPECIAL: bool = true;

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        ));
    }

    if REQUIRE_UPPERCASE && !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if REQUIRE_LOWERCASE && !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if REQUIRE_DIGIT && !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one digit".to_string());
    }

    if REQUIRE_SPECIAL && !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_too_short() {
        let result = validate_password("Short1!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("12 characters"));
    }

    #[test]
    fn test_password_no_uppercase() {
        let result = validate_password("lowercase1234!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("uppercase"));
    }

    #[test]
    fn test_password_no_lowercase() {
        let result = validate_password("UPPERCASE1234!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lowercase"));
    }

    #[test]
    fn test_password_no_digit() {
        let result = validate_password("NoDigitsHere!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("digit"));
    }

    #[test]
    fn test_password_no_special() {
        let result = validate_password("NoSpecialChar1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("special"));
    }

    #[test]
    fn test_password_valid() {
        let result = validate_password("ValidPassword1!");
        assert!(result.is_ok());
    }
}

