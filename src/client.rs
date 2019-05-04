use failure::Fail;
use reqwest::Client as HttpClient;

pub struct Client {
    pub(crate) access_key: String,
    pub(crate) secret: String,
    pub(crate) http: HttpClient,
}

impl Client {
    pub fn new<S: Into<String>>(access_key: S, secret: S) -> Self {
        Client {
            access_key: access_key.into(),
            secret: secret.into(),
            http: HttpClient::new(),
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "http client error")]
    Reqwest(#[fail(cause)] reqwest::Error),

    #[fail(display = "internal error occurred")]
    Internal(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
