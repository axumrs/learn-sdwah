use std::sync::{Arc, LazyLock};

use arc_swap::ArcSwap;
use axum::{
    Form,
    extract::Query,
    http,
    response::{Html, IntoResponse},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub company: Company,
}
pub async fn user_list() -> Html<String> {
    // 数据来源：https://jsonplaceholder.typicode.com/users
    let json_data = include_str!("./users.json");
    let users: Vec<User> = serde_json::from_str(json_data).unwrap();
    let rows_str = users
        .iter()
        .map(|u| {
            format!(
                r#"<tr>
      <td>{id}</td>
      <td>{name}</td>
      <td>{email}</td>
      <td>{company_name}</td>
    </tr>"#,
                id = u.id,
                name = u.name,
                email = u.email,
                company_name = u.company.name
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let html = format!(
        r#"<table class="table">
  <thead>
    <tr>
      <th>ID</th>
      <th>姓名</th>
      <th>邮箱</th>
      <th>公司</th>
    </tr>
  </thead>
  <tbody>
  {rows_str}
  </tbody>
</table>"#
    );
    Html(html)
}

#[derive(Deserialize)]
pub struct InputValidateQuery {
    pub email: Option<String>,
    pub password: Option<String>,
}

const EXISTING_EMAILS: &[&str] = &["foo@bar.com", "bar@foo.com", "use@foo.bar"];
const BAD_PASSWORDS: &[&str] = &["foobar", "123456"];
pub async fn email_validate(Query(q): Query<InputValidateQuery>) -> String {
    let email = q.email.unwrap_or_default();
    _validate_email(&email)
}

fn _validate_email(email: &str) -> String {
    if email.len() < 5 {
        return "邮箱太短".to_string();
    }
    if !email.contains('@') {
        return "邮箱格式不正确".to_string();
    }
    if EXISTING_EMAILS.contains(&email) {
        "邮箱已存在".to_string()
    } else {
        "".to_string()
    }
}

pub async fn password_validate(Query(q): Query<InputValidateQuery>) -> String {
    let password = q.password.unwrap_or_default();
    _validate_password(&password)
}

fn _validate_password(password: &str) -> String {
    if password.len() < 6 {
        return "密码太短".to_string();
    }
    if BAD_PASSWORDS.contains(&password) {
        return "密码太简单".to_string();
    }

    "".into()
}

pub async fn register_user(Form(frm): Form<InputValidateQuery>) -> impl IntoResponse {
    let email = frm.email.unwrap_or_default();
    let password = frm.password.unwrap_or_default();
    let email_validate_msg = _validate_email(&email);
    let password_validate_msg = _validate_password(&password);

    let valid_pass = email_validate_msg.is_empty() && password_validate_msg.is_empty();

    let mut htmls = vec![];
    if valid_pass {
        htmls.push(format!("<span>注册成功</span>"));
    } else {
        htmls.push(format!("<span></span>"));
    }
    if !email_validate_msg.is_empty() {
        htmls.push(format!(r##"<p class="text-sm text-red-600" id="email-error-msg" hx-swap-oob="true">{email_validate_msg}</p>"##));
    }
    if !password_validate_msg.is_empty() {
        htmls.push(format!(r##"<p class="text-sm text-red-600" id="password-error-msg" hx-swap-oob="true">{password_validate_msg}</p>"##));
    }

    let status = if valid_pass {
        http::StatusCode::OK
    } else {
        http::StatusCode::BAD_REQUEST
    };

    (status, Html(htmls.join("\n")))
}

static USERS_FOR_SEARCH: LazyLock<ArcSwap<Vec<&'static str>>> = LazyLock::new(|| {
    ArcSwap::new(Arc::new(vec![
        "Amanda", "Gerri", "Jeremy", "Mark", "Meghan", "Pat", "RC", "Richard", "Tami",
    ]))
});

#[derive(Deserialize)]
pub struct SearchUserQuery {
    pub name: Option<String>,
}
pub async fn search(Form(frm): Form<SearchUserQuery>) -> Html<String> {
    let name = frm.name.unwrap_or_default();
    let users = USERS_FOR_SEARCH.load();
    let html = users
        .iter()
        .filter(|&u| u.to_lowercase().contains(&name.to_lowercase()))
        .map(|u| format!("<li>{u}</li>"))
        .collect::<Vec<_>>()
        .join("");
    Html(html)
}
