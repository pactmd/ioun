use uuid::Uuid;

pub mod account;
pub mod session;

pub enum Unique {
    Id(Uuid),
    Email(String),
}

impl Unique {
    pub fn key(&self) -> String {
        match self {
            Unique::Id(..) => "id".to_string(),
            Unique::Email(..) => "email".to_string(),
        }
    }

    pub fn value(&self) -> String {
        match self {
            Unique::Id(id) => id.to_string(),
            Unique::Email(email) => email.to_string()
        }
    }
}