mod blog_post;
use chrono::{DateTime, Local, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Serializer};


pub(crate) use blog_post::BlogPost;


fn serialize_object_id_option_as_hex_string<S>(
    val: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match val {
        Some(oid) => oid.to_hex().serialize(serializer),
        None => serializer.serialize_none(),
    }
}

mod datetime_format {
    use chrono::{DateTime, Local, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // serialize_with 函数的签名必须遵循以下模式：
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // 尽管也可以对输入类型 T 进行泛型化。
    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => {
                // utc 时间转换为本地时间
                let s = format!("{}", date.with_timezone(&Local).format(FORMAT));
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    // deserialize_with 函数的签名必须遵循以下模式：
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // 尽管也可以对输出类型 T 进行泛型化。
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)))
    }
}

fn get_local_now() -> Option<DateTime<Utc>> {
    Some(Local::now().to_utc())
}


