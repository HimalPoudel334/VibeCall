use regex::Regex;
use serde::Deserialize;

const EMAIL_REGEX_STR: &str = r"^[^\s@]+@[^\s@]+\.[^\s@]+$";
lazy_static::lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(EMAIL_REGEX_STR).unwrap();
}

#[derive(Deserialize)]
#[serde(try_from = "&str")]
pub struct Email {
    email: String,
}

impl TryFrom<&str> for Email {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if EMAIL_REGEX.is_match(s) {
            Ok(Email {
                email: s.to_lowercase().to_string(),
            })
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
