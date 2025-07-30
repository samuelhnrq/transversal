use askama::Template;
use askama_web::WebTemplate;
use models::{ActiveValue, Value, generated::album};

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexPage {
    pub user: Option<models::generated::user::Model>,
}

#[derive(Template, WebTemplate)]
#[template(path = "album_view.html")]
pub struct AlbumView {
    pub album: album::ActiveModel,
    pub albums: Vec<album::Model>,
    pub user: Option<models::generated::user::Model>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[gtest]
    fn renders_album_title() {
        let album = album::ActiveModel {
            title: ActiveValue::Set("A night at the opera".to_string()),
            ..Default::default()
        };
        let value = AlbumView {
            album,
            albums: vec![],
            user: None,
        };
        let rendered = value.render().unwrap();
        expect_that!(rendered, contains_substring("A night at the opera"));
    }
}
