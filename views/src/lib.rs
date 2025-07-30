use askama::Template;
use askama_web::WebTemplate;
use models::{
    ActiveValue, Value,
    generated::user::{self},
};

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexPage {
    pub name: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "user_details.html")]
pub struct UserDetailsPage {
    pub user: user::ActiveModel,
    pub users: Vec<user::Model>,
}

pub(crate) fn is_value_set<T: Into<Value>>(value: &ActiveValue<T>) -> bool {
    matches!(
        value,
        models::ActiveValue::Set(_) | models::ActiveValue::Unchanged(_)
    )
}

pub(crate) mod filters {
    use models::{ActiveValue, Value};

    #[allow(
        clippy::unnecessary_wraps,
        reason = "askama requires a Result return type"
    )]
    pub fn maybe<T: Into<Value> + ToString>(
        value: &ActiveValue<T>,
        _: &dyn askama::Values,
    ) -> askama::Result<String> {
        match value {
            ActiveValue::Set(val) => Ok(val.to_string()),
            ActiveValue::NotSet => Ok(String::new()),
            ActiveValue::Unchanged(old) => Ok(old.to_string()),
        }
    }
}
