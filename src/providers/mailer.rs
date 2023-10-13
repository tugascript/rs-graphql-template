// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};

use crate::common::{ServiceError, SOMETHING_WENT_WRONG};

#[derive(Clone, Debug)]
pub struct Mailer {
    email: String,
    front_end_url: String,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl Mailer {
    pub fn new() -> Self {
        let host = env::var("EMAIL_HOST").unwrap();
        let email = env::var("EMAIL_USER").unwrap();
        let password = env::var("EMAIL_PASSWORD").unwrap();
        let port = env::var("EMAIL_PORT").unwrap().parse::<u16>().unwrap();
        let front_end_url = env::var("FRONT_END_URL").unwrap();
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .unwrap()
            .port(port)
            .credentials(Credentials::new(email.to_owned(), password))
            .build();

        Self {
            email,
            front_end_url,
            mailer,
        }
    }

    fn send_email(&self, to: String, subject: String, body: String) -> Result<(), ServiceError> {
        let message = Message::builder()
            .from(self.email.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body);

        match message {
            Ok(msg) => {
                let master_mailer = self.mailer.clone();
                tokio::spawn(async move {
                    match master_mailer.send(msg).await {
                        Err(_) => eprintln!("Error sending the email"),
                        _ => (),
                    }
                });
                Ok(())
            }
            Err(e) => Err(ServiceError::internal_server_error(
                SOMETHING_WENT_WRONG,
                Some(e),
            )),
        }
    }

    pub fn send_confirmation_email(
        &self,
        email: &str,
        full_name: &str,
        jwt: &str,
    ) -> Result<(), ServiceError> {
        tracing::trace_span!("Sending confirmation email");
        let link = format!("{}/confirmation/{}", self.front_end_url, &jwt);

        self.send_email(
            email.to_owned(),
            format!("Email confirmation, {}", full_name),
            format!(
                r#"
            <bodies>
              <p>Hello {},</p>
              <br />
              <p>Welcome to Your Company,</p>
              <p>
                Click
                <b>
                  <a href='{}' target='_blank'>here</a>
                </b>
                to activate your acount or go to this link:
                {}
              </p>
              <p><small>This link will expire in an hour.</small></p>
              <br />
              <p>Best regards,</p>
              <p>Your Company Team</p>
            </bodies>
          "#,
                full_name, &link, &link,
            ),
        )
    }

    pub fn send_access_email(
        &self,
        email: &str,
        full_name: &str,
        code: &str,
    ) -> Result<(), ServiceError> {
        self.send_email(
            email.to_owned(),
            format!("Your access code, {}", full_name),
            format!(
                r#"
                <bodies>
                    <p>Hello {},</p>
                    <br />
                    <p>Welcome to Your Company,</p>
                    <p>
                        Your access code is
                        <b>{}</b>
                    </p>
                    <p><small>This code will expire in 15 minutes.</small></p>
                    <br />
                    <p>Best regards,</p>
                    <p>Your Company Team</p>
                </bodies> 
            "#,
                full_name, code
            ),
        )
    }

    pub fn send_password_reset_email(
        &self,
        email: &str,
        full_name: &str,
        token: &str,
    ) -> Result<(), ServiceError> {
        let link = format!("{}/confirmation/{}", self.front_end_url, &token);

        self.send_email(
            email.to_owned(),
            format!("Email confirmation, {}", full_name),
            format!(
                r#"
                <bodies>
                    <p>Hello {},</p>
                    <br />
                    <p>Your password reset link:
                    <b><a href='{}' target='_blank'>here</a></b></p>
                    <p>Or go to this link: {}</p>
                    <p><small>This link will expire in 30 minutes.</small></p>
                    <br />
                    <p>Best regards,</p>
                    <p>Your Company Team</p>
                </bodies>
                "#,
                &full_name, &link, &link,
            ),
        )
    }
}
