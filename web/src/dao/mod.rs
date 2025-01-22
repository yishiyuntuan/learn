use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson};
use mongodb::{bson, Client, Collection, Database};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use springboot::application::get_service;
use springboot::component::service::Service;
use springboot::config::Configurable;
use springboot_logger::{debug, info};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::LazyLock;

pub(crate) mod post_mapper;

#[derive(Clone, Configurable, Deserialize, Debug)]
#[config_prefix = "mongo"]
struct MongoConfig {
    host: String,
    port: String,
    user: String,
    password: String,
    db_name: String,
}

async fn mongo_client(cfg: &MongoConfig) -> Database {
    let uri = format!(
        "mongodb://{}:{}@{}:{}",
        cfg.user, cfg.password, cfg.host, cfg.port
    );
    let client = Client::with_uri_str(uri).await.unwrap();
    client.database(cfg.db_name.as_str())
}

#[derive(Clone, Service)]
pub(crate) struct MongoService {
    #[inject(func = Self::init_db(&config))]
    pub(crate) db: Database,

    #[inject(config)]
    config: MongoConfig,
}

impl MongoService {
    fn init_db(config: &MongoConfig) -> Database {
        debug!("mongo 连接中 连接信息：{config:?}");
        futures::executor::block_on(mongo_client(config))
    }
}

static MONGO: LazyLock<MongoService> = LazyLock::new(get_service::<MongoService>);

pub fn get_collection<T>() -> DataBase<Collection<T>>
where
    T: Send + Sync,
{
    let name = std::any::type_name::<T>();
    // 按照::分割，取最后一个
    let name = name.split("::").last().unwrap();
    // name转换为小写
    let collection_name = name.to_lowercase();
    let collection = MONGO.db.collection::<T>(collection_name.as_str());
    info!("init collection: {}", collection_name);
    DataBase(collection)
}

pub struct DataBase<T>(pub T);

impl<T> Deref for DataBase<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// orm 基础接口
pub trait BaseMapper<T> {
    async fn query_by_id(&self, id: &str) -> anyhow::Result<T>;
    async fn save(&self, t: &T) -> anyhow::Result<ObjectId>;
    async fn update_or_save(&self, t: &T) -> anyhow::Result<ObjectId>;
    async fn delete_by_id(&self, id: &str) -> anyhow::Result<()>;
    async fn update_by_id(&self, id: &str, t: &T) -> anyhow::Result<()>;
}

impl<T> BaseMapper<T> for DataBase<Collection<T>>
where
    T: Sync + Send + DeserializeOwned + Serialize + Debug + Clone,
{
    async fn query_by_id(&self, id: &str) -> anyhow::Result<T> {
        self.find_one(doc! { "_id": ObjectId::parse_str(id)? })
            .await?
            .ok_or(anyhow::anyhow!("not found"))
    }

    async fn save(&self, t: &T) -> anyhow::Result<ObjectId> {
        self.insert_one(t)
            .await?
            .inserted_id
            .as_object_id()
            .ok_or(anyhow::anyhow!("convert error"))
    }

    async fn update_or_save(&self, t: &T) -> anyhow::Result<ObjectId> {
        let mut document = bson::to_document(t)?;
        let id = document
            .remove("id")
            .or_else(|| Some(Bson::from(ObjectId::new().to_hex())))
            .ok_or(anyhow::anyhow!("id not found"))?;

        let id = ObjectId::parse_str(id.as_str().ok_or(anyhow::anyhow!("id not found"))?)?;
        info!("{:?}", id);

        let res = self
            .update_one(doc! {"_id":id}, doc! { "$set": document })
            .upsert(true)
            .await?;
        info!("{:?}", res);

        // 如果没有upserted_id，说明是更新
        if res.upserted_id.is_none() {
            Ok(id)
        } else {
            Ok(res
                .upserted_id
                .ok_or(anyhow::anyhow!("upsert fail"))?
                .as_object_id()
                .ok_or(anyhow::anyhow!("convert error"))?)
        }
    }

    async fn delete_by_id(&self, id: &str) -> anyhow::Result<()> {
        let result = self
            .delete_one(doc! {"_id":ObjectId::parse_str(id)?})
            .await?;
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("delete fail"));
        }
        Ok(())
    }

    async fn update_by_id(&self, id: &str, t: &T) -> anyhow::Result<()> {
        let update = doc! { "$set":  bson::to_document(t)? };
        let result = self
            .update_one(doc! {"_id":ObjectId::parse_str(id)?}, update)
            .await?;
        if result.modified_count == 0 {
            return Err(anyhow::anyhow!("update fail"));
        }
        Ok(())
    }
}
