use crate::dao::{get_collection, BaseMapper, DataBase};
use crate::model::BlogPost;
use mongodb::Collection;
use springboot_logger::info;
use std::sync::LazyLock;

// 小写不告警
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
static blog_mapper: LazyLock<DataBase<Collection<BlogPost>>> =
    LazyLock::new(|| get_collection::<BlogPost>());

pub(crate) async fn query(id: String) -> anyhow::Result<BlogPost> {
    blog_mapper.query_by_id(&id).await
}

pub(crate) async fn save_post(post: BlogPost) -> anyhow::Result<BlogPost> {
    let id = blog_mapper.save(&post).await?;
    let mut post = post.clone();
    info!("{id:?}");
    // post.set_id(id);
    // post.id = Some(id);
    post.set_id(Some(id));
    Ok(post)
}

pub(crate) async fn update_or_save(mut p: BlogPost) -> anyhow::Result<BlogPost> {
    let id = blog_mapper.update_or_save(&p).await?;
    p.set_id(Some(id));
    Ok(p)
}
