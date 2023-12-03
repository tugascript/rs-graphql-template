use std::env;

use uuid::Uuid;

use super::Environment;

type HOST = String;
type PORT = u16;

pub struct ServerLocation(pub HOST, pub PORT);

impl ServerLocation {
    pub fn new() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
        Self(host, port)
    }
}

pub struct ApiURLs {
    pub api_id: String,
    pub backend_url: String,
    pub frontend_url: String,
}

impl ApiURLs {
    pub fn new(environment: &Environment, port: u16) -> Self {
        let api_id = env::var("API_ID").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => panic!("Missing the API_ID environment variable."),
        });
        let backend_url = env::var("BACKEND_URL").unwrap_or_else(|_| match environment {
            Environment::Development => format!("http://localhost:{}", port),
            Environment::Production => panic!("Missing the BACKEND_URL environment variable."),
        });
        let frontend_url =
            env::var("FRONTEND_URL").expect("Missing the FRONTEND_URL environment variable.");

        Self {
            api_id,
            backend_url,
            frontend_url,
        }
    }
}
