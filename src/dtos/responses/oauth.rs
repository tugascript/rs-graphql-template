use serde::{Deserialize, Serialize};

pub struct UserInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub date_of_birth: String,
    pub picture: Option<String>,
}

impl TryFrom<GoogleUserInfoResponse> for UserInfo {
    type Error = String;

    fn try_from(value: GoogleUserInfoResponse) -> Result<Self, Self::Error> {
        let first_name = value
            .given_name
            .ok_or_else(|| "Missing given_name".to_string())?;
        let last_name = value
            .family_name
            .ok_or_else(|| "Missing family_name".to_string())?;
        let email = value.email.ok_or_else(|| "Missing email".to_string())?;
        let date_of_birth = value
            .birthdate
            .ok_or_else(|| "Missing birthdate".to_string())?;

        Ok(Self {
            first_name,
            last_name,
            email,
            date_of_birth,
            picture: value.picture,
        })
    }
}

impl TryFrom<FacebookUserInfoResponse> for UserInfo {
    type Error = String;

    fn try_from(value: FacebookUserInfoResponse) -> Result<Self, Self::Error> {
        let first_name = value
            .first_name
            .ok_or_else(|| "Missing first_name".to_string())?;
        let last_name = value
            .last_name
            .ok_or_else(|| "Missing last_name".to_string())?;
        let email = value.email.ok_or_else(|| "Missing email".to_string())?;
        let birth_date = value
            .birthday
            .ok_or_else(|| "Missing birthday".to_string())?;

        Ok(Self {
            first_name,
            last_name,
            email,
            date_of_birth: birth_date,
            picture: value.picture.and_then(|p| p.data).and_then(|d| d.url),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfoResponse {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub locale: Option<String>,
    pub birthdate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookPictureInfo {
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub url: Option<String>,
    pub is_silhouette: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookPictureData {
    pub data: Option<FacebookPictureInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookUserInfoResponse {
    pub id: String,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub birthday: Option<String>,
    pub picture: Option<FacebookPictureData>,
    pub gender: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OAuthUserInfo {
    Google(GoogleUserInfoResponse),
    Facebook(FacebookUserInfoResponse),
}

impl TryInto<UserInfo> for OAuthUserInfo {
    type Error = String;

    fn try_into(self) -> Result<UserInfo, Self::Error> {
        match self {
            OAuthUserInfo::Google(google) => google.try_into(),
            OAuthUserInfo::Facebook(facebook) => facebook.try_into(),
        }
    }
}
