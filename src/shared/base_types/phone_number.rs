use regex::Regex;
use serde::Deserialize;

lazy_static::lazy_static! {
    static ref PHONE_NUMBER_REGEX: Regex =
        Regex::new(r"^(?:\+?977)?9[78]\d{8}$").unwrap();
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct PhoneNumber {
    phone_number: String,
}

impl TryFrom<String> for PhoneNumber {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if PHONE_NUMBER_REGEX.is_match(s.as_str()) {
            Ok(PhoneNumber { phone_number: s })
        } else {
            Err("Invalid phone number format")
        }
    }
}

// impl FromStr for PhoneNumber {
//     type Err = &'static str;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if PHONE_NUMBER_REGEX.is_match(s) {
//             Ok(PhoneNumber {
//                 phone_number: s.to_string(),
//             })
//         } else {
//             Err("Invalid phone number format")
//         }
//     }
// }

impl PhoneNumber {
    pub fn get_number(&self) -> &str {
        &self.phone_number
    }
}
