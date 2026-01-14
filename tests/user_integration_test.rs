//! Integration tests for the User domain entity
//!
//! These tests verify the complete lifecycle of a User,
//! including registration, authentication, profile updates, and email changes.

use mini_rust_api::domain::user::{Email, User};

/// Test the complete user registration and authentication flow
#[test]
fn test_user_registration_and_authentication_flow() {
    // Step 1: Create a valid email
    let email =
        Email::try_from("john.doe@example.com".to_string()).expect("Valid email should be created");

    // Step 2: Register a new user with all required information
    let user = User::register(
        email,
        "SecurePassword123".to_string(),
        "John".to_string(),
        "Doe".to_string(),
        30,
    )
    .expect("User registration should succeed");

    // Step 3: Verify user was created without an ID (not persisted yet)
    assert!(user.id().is_none(), "New user should not have an ID");

    // Step 4: Verify email was properly normalized
    assert_eq!(user.email().as_ref(), "john.doe@example.com");

    // Step 5: Verify profile data
    assert_eq!(user.profile().first_name(), "John");
    assert_eq!(user.profile().last_name(), "Doe");
    assert_eq!(user.profile().age(), 30);
    assert_eq!(user.profile().full_name(), "John Doe");

    // Step 6: Test successful authentication
    assert!(
        user.authenticate("SecurePassword123").is_ok(),
        "Authentication with correct password should succeed"
    );

    // Step 7: Test failed authentication
    assert!(
        user.authenticate("WrongPassword123").is_err(),
        "Authentication with wrong password should fail"
    );
}

/// Test user profile modifications
#[test]
fn test_user_profile_modification_flow() {
    let email = Email::try_from("jane.smith@example.com".to_string()).unwrap();
    let mut user = User::register(
        email,
        "ValidPass123".to_string(),
        "Jane".to_string(),
        "Smith".to_string(),
        25,
    )
    .unwrap();

    // Verify initial profile
    assert_eq!(user.profile().full_name(), "Jane Smith");
    assert_eq!(user.profile().age(), 25);

    // Update the profile
    user.update_profile("Janet".to_string(), "Johnson".to_string(), 26)
        .expect("Profile update should succeed");

    // Verify updated profile
    assert_eq!(user.profile().first_name(), "Janet");
    assert_eq!(user.profile().last_name(), "Johnson");
    assert_eq!(user.profile().age(), 26);
    assert_eq!(user.profile().full_name(), "Janet Johnson");
}

/// Test user email change flow
#[test]
fn test_user_email_change_flow() {
    let initial_email = Email::try_from("original@example.com".to_string()).unwrap();
    let mut user = User::register(
        initial_email,
        "SecurePass123".to_string(),
        "Test".to_string(),
        "User".to_string(),
        21,
    )
    .unwrap();

    assert_eq!(user.email().as_ref(), "original@example.com");

    // Change to a new email
    let new_email = Email::try_from("updated@example.com".to_string()).unwrap();
    user.change_email(new_email)
        .expect("Email change should succeed");

    assert_eq!(user.email().as_ref(), "updated@example.com");

    // Changing to the same email should be a no-op (not an error)
    let same_email = Email::try_from("updated@example.com".to_string()).unwrap();
    user.change_email(same_email)
        .expect("Changing to same email should succeed");

    assert_eq!(user.email().as_ref(), "updated@example.com");
}

/// Test password change flow
#[test]
fn test_user_password_change_flow() {
    let email = Email::try_from("password.test@example.com".to_string()).unwrap();
    let mut user = User::register(
        email,
        "OriginalPass123".to_string(),
        "Password".to_string(),
        "Tester".to_string(),
        35,
    )
    .unwrap();

    // Verify original password works
    assert!(user.authenticate("OriginalPass123").is_ok());

    // Change the password
    user.change_password("NewSecurePass456".to_string())
        .expect("Password change should succeed");

    // Old password should no longer work
    assert!(
        user.authenticate("OriginalPass123").is_err(),
        "Old password should fail after change"
    );

    // New password should work
    assert!(
        user.authenticate("NewSecurePass456").is_ok(),
        "New password should work after change"
    );
}

/// Test validation errors in the user registration flow
#[test]
fn test_user_registration_validation_errors() {
    // Test with invalid email
    let invalid_email = Email::try_from("not-an-email".to_string());
    assert!(invalid_email.is_err(), "Invalid email should be rejected");

    // Test with weak password (no uppercase)
    let email = Email::try_from("test@example.com".to_string()).unwrap();
    let weak_password_result = User::register(
        email.clone(),
        "weakpassword123".to_string(),
        "Test".to_string(),
        "User".to_string(),
        25,
    );
    assert!(
        weak_password_result.is_err(),
        "Weak password should be rejected"
    );

    // Test with short password
    let email = Email::try_from("test2@example.com".to_string()).unwrap();
    let short_password_result = User::register(
        email,
        "Short1".to_string(),
        "Test".to_string(),
        "User".to_string(),
        25,
    );
    assert!(
        short_password_result.is_err(),
        "Short password should be rejected"
    );

    // Test with empty first name
    let email = Email::try_from("test3@example.com".to_string()).unwrap();
    let empty_name_result = User::register(
        email,
        "ValidPass123".to_string(),
        "".to_string(),
        "User".to_string(),
        25,
    );
    assert!(
        empty_name_result.is_err(),
        "Empty first name should be rejected"
    );

    // Test with user too young
    let email = Email::try_from("test4@example.com".to_string()).unwrap();
    let underage_result = User::register(
        email,
        "ValidPass123".to_string(),
        "Test".to_string(),
        "User".to_string(),
        17,
    );
    assert!(underage_result.is_err(), "Underage user should be rejected");
}

/// Test email normalization across the flow
#[test]
fn test_email_normalization_in_user_flow() {
    // Email with mixed case and whitespace
    let email = Email::try_from("  JOHN.DOE@EXAMPLE.COM  ".to_string())
        .expect("Email with whitespace should be normalized");

    let user = User::register(
        email,
        "SecurePass123".to_string(),
        "John".to_string(),
        "Doe".to_string(),
        25,
    )
    .unwrap();

    // Email should be lowercase and trimmed
    assert_eq!(user.email().as_ref(), "john.doe@example.com");
}
