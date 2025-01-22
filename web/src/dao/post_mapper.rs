use crate::dao::{BaseMapper, DataBase};
use crate::model::BlogPost;
use mongodb::bson::doc;
use mongodb::Collection;

// dao层接口
pub trait PostMapper: BaseMapper<BlogPost> {
    async fn get_post(&self, id: &str) -> BlogPost;
    async fn save_post(&self, post: BlogPost) -> BlogPost;
}

impl PostMapper for DataBase<Collection<BlogPost>> {
    async fn get_post(&self, id: &str) -> BlogPost {
        self.find_one(doc! { "title": id }).await.unwrap().unwrap()
    }

    async fn save_post(&self, post: BlogPost) -> BlogPost {
        // mongodb::bson::serde_helpers::serialize_hex_string_as_object_id();
        todo!()
    }
}
