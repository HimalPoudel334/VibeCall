use regex::Regex;
use serde::Deserialize;

const EMAIL_REGEX_STR: &str = r"^[^\s@]+@[^\s@]+\.[^\s@]+$";
lazy_static::lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(EMAIL_REGEX_STR).unwrap();
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Email {
    email: String,
}

impl TryFrom<String> for Email {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if EMAIL_REGEX.is_match(s.as_str()) {
            Ok(Email { email: s })
        } else {
            Err("Invalid email format")
        }
    }
}

impl Email {
    pub fn get_email(&self) -> &str {
        &self.email
    }
}
