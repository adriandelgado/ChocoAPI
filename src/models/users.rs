use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::erro::add_error;

/// A domain user.
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub full_name: Option<String>,
    pub profile_pic_id: Option<Uuid>,
    pub email_id: Uuid,
    pub passwd_hash: String,
    pub active: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Represents a user to be inserted in the database.
pub struct InsertableUser {
    username: String,
    full_name: Option<String>,
    profile_pic_id: Option<Uuid>,
    email_id: Uuid,
    passwd_hash: String,
}

impl InsertableUser {
    #[must_use]
    pub fn username(&self) -> String {
        self.username.clone()
    }

    #[must_use]
    pub fn full_name(&self) -> Option<String> {
        self.full_name.clone()
    }

    #[must_use]
    pub fn profile_pic_id(&self) -> Option<Uuid> {
        self.profile_pic_id
    }

    #[must_use]
    pub fn email_id(&self) -> Uuid {
        self.email_id
    }

    #[must_use]
    pub fn passwd_hash(&self) -> String {
        self.passwd_hash.clone()
    }
}

/// Build a new `InsertableUser`.
pub struct InsertableUserBuilder {
    username: String,
    full_name: Option<String>,
    profile_pic_id: Option<Uuid>,
    email_id: Option<Uuid>,
    passwd_hash: String,
}

impl InsertableUserBuilder {
    #[must_use]
    pub fn new() -> Self {
        InsertableUserBuilder {
            username: String::default(),
            full_name: None,
            profile_pic_id: None,
            email_id: None,
            passwd_hash: String::default(),
        }
    }

    #[must_use]
    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    #[must_use]
    pub fn with_full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    #[must_use]
    pub fn with_password(mut self, passwd: String) -> Self {
        // TODO: hash password
        self.passwd_hash = passwd;
        self
    }

    #[must_use]
    pub fn with_email_id(mut self, email_id: Uuid) -> Self {
        self.email_id = Some(email_id);
        self
    }

    #[must_use]
    pub fn with_profile_pic_id(mut self, profile_pic_id: Uuid) -> Self {
        self.profile_pic_id = profile_pic_id.into();
        self
    }

    /// Build a new `InsertableUser` from this `InsertableUserBuilder`.
    /// # Panics
    /// - never
    pub fn build(self) -> Result<InsertableUser, HashMap<String, Vec<String>>> {
        let mut errors = HashMap::new();

        if self.username.is_empty() {
            errors = add_error(errors, "username".to_string(), "Missing field".to_string());
        }

        if self.passwd_hash.is_empty() {
            errors = add_error(errors, "password".to_string(), "Missing field".to_string());
        }

        if self.email_id.is_none() {
            errors = add_error(errors, "email".to_string(), "Missing field".to_string());
        }

        if errors.is_empty() {
            Ok(InsertableUser {
                username: self.username,
                full_name: self.full_name,
                profile_pic_id: self.profile_pic_id,
                email_id: self.email_id.unwrap(),
                passwd_hash: self.passwd_hash,
            })
        } else {
            Err(errors)
        }
    }
}

impl Default for InsertableUserBuilder {
    fn default() -> Self {
        Self::new()
    }
}
