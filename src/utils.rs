use crate::deserializers::question::Question;
use crate::errors::AppResult;
use crate::graphql::problemset_question_list::Query as QuestionQuery;
use crate::graphql::GQLLeetcodeQuery;
use crate::{config::Config, db_ops::ModelUtils};
use sea_orm::DatabaseConnection;

pub async fn update_database_questions(
    client: &reqwest::Client,
    database_client: &DatabaseConnection,
) -> AppResult<()> {
    let query = QuestionQuery::default();
    let query_response = query.post(&client).await?;
    let total_questions = query_response.get_total_questions();

    let chunk_size = 100;
    let n_chunks = total_questions / chunk_size;
    for i in kdam::tqdm!(0..n_chunks) {
        let skip = i * chunk_size;
        let take = chunk_size;
        let client_copy = client.clone();
        let db_client_copy = database_client.clone();
        let resp = QuestionQuery::new(take, skip).post(&client_copy).await?;
        let questions = resp.get_questions();
        Question::multi_insert(&db_client_copy, questions).await?;
    }

    if total_questions % chunk_size != 0 {
        let skip = n_chunks * chunk_size;
        let take = total_questions - skip;
        let client_copy = client.clone();
        let db_client_copy = database_client.clone();
        let resp = QuestionQuery::new(take, skip).post(&client_copy).await?;
        Question::multi_insert(&db_client_copy, resp.get_questions()).await?;
    }
    Ok(())
}

use crate::migrations::{Migrator, MigratorTrait};

pub async fn do_migrations(database_client: &DatabaseConnection) -> AppResult<()> {
    Ok(Migrator::up(database_client, None).await?)
}

use reqwest::header::{HeaderMap, HeaderValue};

pub async fn get_reqwest_client(config: &Config) -> AppResult<reqwest::Client> {
    let csrf = config.leetcode.csrftoken.as_str();
    let sess = config.leetcode.leetcode_session.as_str();
    let mut headers = HeaderMap::new();
    let header_k_v = [
        (
            "Cookie",
            format!("LEETCODE_SESSION={sess}; csrftoken={csrf}"),
        ),
        ("Content-Type", format!("application/json")),
        ("x-csrftoken", format!("{csrf}")),
        ("Origin", format!("https://leetcode.com")),
        ("Referer", format!("https://leetcode.com")),
        ("Connection", format!("keep-alive")),
    ];

    for (key, value) in header_k_v {
        headers.append(key, HeaderValue::from_str(value.as_str())?);
    }

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
    Ok(client)
}

use crate::config::Db;

pub async fn get_config() -> AppResult<Option<Config>> {
    let config_path = Config::get_base_config()?;
    let config: Config;

    if !config_path.exists() {
        config = Config::default();
        config.write_config(Config::get_base_config()?).await?;
        println!("\nConfig is created at config_path {}.\n Kindly set LEETCODE_SESSION and csrftoken in the config file. These can be obained from leetcode cookies in the browser.", config_path.display());
        let db_data_path = Db::get_base_sqlite_data_path()?;
        if !db_data_path.exists() {
            Db::touch_default_db().await?;
            println!("\nDatabase resides in {}", db_data_path.display());
        }
        return Ok(None);
    } else {
        println!("Config file found @ {}", &config_path.display());
        config = Config::read_config(config_path).await?;
        Ok(Some(config))
    }
}
