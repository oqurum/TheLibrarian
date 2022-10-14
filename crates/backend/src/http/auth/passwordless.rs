// You supply an email. We email the link to authenticate with.

// TODO: Better security. Simple Proof of Concept.

use actix_identity::Identity;
use actix_web::{http::header, HttpResponse};
use actix_web::{web, HttpRequest};
use common::api::{ApiErrorResponse, WrappingResponse};
use common_local::{ConfigEmail, Permissions};

use crate::config::get_config;
use crate::http::JsonResponse;
use crate::model::{AuthModel, MemberModel, NewMemberModel};
use crate::{Error, Result, WebResult};
use chrono::Utc;
use lettre::message::header::ContentType;
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::prelude::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub static PASSWORDLESS_PATH: &str = "/auth/passwordless";
pub static PASSWORDLESS_PATH_CB: &str = "/auth/passwordless/response";

#[derive(Serialize, Deserialize)]
pub struct PostPasswordlessCallback {
    pub email: String,
}

pub async fn post_passwordless_oauth(
    req: HttpRequest,
    query: web::Json<PostPasswordlessCallback>,
    identity: Identity,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<String>> {
    if identity.identity().is_some() {
        return Err(ApiErrorResponse::new("Already logged in").into());
    }

    let config = get_config();

    let (email_config, host) = match (
        config.email,
        req.headers().get("host").and_then(|v| v.to_str().ok()),
    ) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            return Err(
                ApiErrorResponse::new("Missing email from config OR unable to get host").into(),
            )
        }
    };

    let proto = config.server.is_secure.then(|| "https").unwrap_or("http");

    if !get_config().auth.new_users
        && MemberModel::get_by_email(query.0.email.trim(), &db)
            .await?
            .is_none()
    {
        return Err(ApiErrorResponse::new("New user creation is disabled").into());
    }

    let oauth_token = gen_sample_alphanumeric(32, &mut rand::thread_rng());

    let auth_url = format!(
        "{proto}://{host}{}?{}",
        PASSWORDLESS_PATH_CB,
        serde_urlencoded::to_string(QueryCallback {
            oauth_token: oauth_token.clone(),
            email: query.email.clone()
        })
        .map_err(Error::from)?
    );

    let main_html = render_email(
        proto,
        host,
        &email_config.display_name,
        PASSWORDLESS_PATH_CB,
    );

    let auth = AuthModel {
        oauth_token,
        // TODO:
        oauth_token_secret: String::new(),
        created_at: Utc::now(),
    };

    auth.insert(&db).await?;

    send_auth_email(query.0.email, auth_url, main_html, &email_config)?;

    Ok(web::Json(WrappingResponse::okay(String::from("success"))))
}

#[derive(Serialize, Deserialize)]
pub struct QueryCallback {
    pub oauth_token: String,
    pub email: String,
}

pub async fn get_passwordless_oauth_callback(
    query: web::Query<QueryCallback>,
    identity: Identity,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<HttpResponse> {
    if identity.identity().is_some() {
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, "/"))
            .finish());
    }

    let QueryCallback { oauth_token, email } = query.into_inner();

    if AuthModel::remove_by_oauth_token(&oauth_token, &db).await? {
        // Create or Update User.
        let member = if let Some(value) = MemberModel::get_by_email(&email, &db).await? {
            value
        } else {
            let new_member = NewMemberModel {
                // TODO: Strip email
                name: email.clone(),
                email: Some(email),
                password: None,
                permissions: Permissions::basic(),
                localsettings: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            new_member.insert(&db).await?
        };

        super::remember_member_auth(member.id, &identity)?;
    }

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, "/"))
        .finish())
}

pub fn send_auth_email(
    sending_to_email: String,
    alt_text: String,
    main_html: String,
    email_config: &ConfigEmail,
) -> Result<()> {
    let email = Message::builder()
        .from(
            format!(
                "{} <{}>",
                email_config.display_name, email_config.sending_email
            )
            .parse()?,
        )
        .reply_to(email_config.sending_email.parse()?)
        .to(sending_to_email.parse()?)
        .subject(&email_config.subject_line)
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(alt_text),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(main_html),
                ),
        )?;

    let creds = Credentials::new(
        email_config.smtp_username.clone(),
        email_config.smtp_password.clone(),
    );

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&email_config.smtp_relay)?
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)?;

    Ok(())
}

pub fn gen_sample_alphanumeric(amount: usize, rng: &mut ThreadRng) -> String {
    rng.sample_iter(rand::distributions::Alphanumeric)
        .take(amount)
        .map(char::from)
        .collect()
}

fn render_email(
    website_url_protocol: &str,
    website_http_base_host: &str,
    email_display_name: &str,
    email_callback_url: &str,
) -> String {
    format!(
        r#"
        <!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
        <html xmlns="http://www.w3.org/1999/xhtml">
            <head>
                <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
            </head>
            <body
                leftmargin="0"
                marginwidth="0"
                topmargin="0"
                marginheight="0"
                offset="0"
                style="margin: 0; padding: 0; font-family: Verdana, sans-serif; height: 100%;"
            >
                <center>
                    <table style="width: 600px; mso-table-lspace: 0pt; mso-table-rspace: 0pt; margin: 0; padding: 0; font-family: Verdana,sans-serif; border-collapse: collapse; height: 100%;" align="center" border="0" cellpadding="0" cellspacing="0" height="100%" width="100%" id="bodyTable">
                        <tr>
                            <td
                                align="center"
                                valign="top"
                                style="mso-table-lspace: 0pt; mso-table-rspace: 0pt; margin: 0; padding: 20px; font-family: Verdana, sans-serif; height: 100%;"
                            >
                                <div>
                                    <p style="text-align: center; margin-bottom: 30px;">
                                        <img
                                            src="{website_url_protocol}://{website_http_base_host}/brand.png"
                                            width="120"
                                            alt="{email_display_name}"
                                            style="-ms-interpolation-mode: bicubic; border: 0; height: auto; line-height: 100%; outline: none; text-decoration: none;"
                                        >
                                    </p>

                                    <p style="font-size: 1.2em; line-height: 1.3;">Please click and confirm that you want to sign in to {email_display_name}. This link will expire shortly.</p>

                                    <div style="text-align: center;">
                                        <a
                                            style="text-transform: uppercase; letter-spacing: 1px; color: #ffffff; text-decoration: none; display: inline-block; min-height: 48px; line-height: 48px; padding-top: 0; padding-right: 26px; padding-bottom: 0; margin: 20px 0; padding-left: 26px; border: 0; outline: 0; font-size: 14px; font-style: normal; font-weight: 400; text-align: center; white-space: nowrap; border-radius: 3px; text-overflow: ellipsis; max-width: 280px; overflow: hidden; background: white; color: #333132; border: 1px solid #7c7622;"
                                            href="{email_callback_url}"
                                        >Sign in to {email_display_name}</a>
                                    </div>

                                    <p>Or sign in using this link:</p>

                                    <p>
                                        <a
                                            style="font-size: 12px; color: #5c581c; text-decoration: none; word-break: break-all;"
                                            href="{email_callback_url}"
                                        >{email_callback_url}</a>
                                    </p>

                                    <br>

                                    <span>Thanks!</span>

                                    <br>

                                    <strong>{email_display_name}</strong>

                                    <br><br>

                                    <hr style="border: 2px solid #e3e7ec; border-bottom: 0; margin: 20px 0;">

                                    <p style="text-align: center; color: #5c581c;">If you did not make this request, you can safely ignore this e-mail.</p>
                                </div>
                            </td>
                        </tr>
                    </table>
                </center>
            </body>
        </html>"#
    )
}
