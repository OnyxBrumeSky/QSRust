use serde::Deserialize;


#[derive(Deserialize)]
pub struct  IAM {
    pub access_token : String,
    refresh_token : String,
    expires_in : u64,
    expiration : u64,
}


impl Default for IAM {
    fn default() -> Self {
        IAM {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_in: 0,
            expiration: 0,
        }
    }
}


