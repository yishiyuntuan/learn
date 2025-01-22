use bon::Builder;
use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(
    Builder, Getters, Setters, MutGetters, CopyGetters, Serialize, Deserialize, Debug, Clone,
)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub(crate) struct BlogPost {
    #[serde(
        alias = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "super::serialize_object_id_option_as_hex_string"
    )]
    id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "super::datetime_format",
        default = "super::get_local_now"
    )]
    created_at: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "super::datetime_format",
        default = "super::get_local_now"
    )]
    updated_at: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "super::datetime_format",
        default = "super::get_local_now"
    )]
    deleted_at: Option<DateTime<Utc>>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use mongodb::bson;
    use sonic_rs::json;

    #[test]
    fn test() {
        println!("utc {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        println!("local {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
        let mut post = BlogPost::builder()
            .title("aaa".to_string())
            .deleted_at(Utc::now())
            .build();

        post.set_id(Some(ObjectId::new()));

        let document = bson::to_document(&post).unwrap();
        println!("{:?}", document);

        let value = json!(post);

        println!("{value:?}");
        let doc = bson::to_document(&post).unwrap();
        println!("{doc:?}");

        let json = r#"{}"#;
        let post1 = serde_json::from_str::<BlogPost>(json).unwrap();
        println!("{:?}", post1);
        println!("{}", json!(post1));
    }
}