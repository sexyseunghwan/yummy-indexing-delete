use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
pub struct ElasticServerConfig {
    pub elastic_host: Vec<String>,
    pub elastic_id: Option<String>,
    pub elastic_pw: Option<String>,
    pub elastic_pool_cnt: i32,
}

impl ElasticServerConfig {
    pub fn new() -> Self {
        let elastic_host: Vec<String> = env::var("ES_DB_URL")
            .expect("[ENV file read Error][initialize_db_clients()] 'ES_DB_URL' must be set")
            .split(",")
            .map(|s| s.to_string())
            .collect();

        let elastic_id: String = env::var("ES_ID")
            .expect("[ENV file read Error][initialize_db_clients()] 'ES_ID' must be set");
        let elastic_pw: String = env::var("ES_PW")
            .expect("[ENV file read Error][initialize_db_clients()] 'ES_PW' must be set");
        let elastic_pool_cnt = env::var("ES_PW")
            .expect("[ENV file read Error][initialize_db_clients()] 'ES_POOL_CNT' must be set")
            .parse::<i32>()
            .expect("[Error][ElasticServerConfig->new] ");

        Self {
            elastic_host,
            elastic_id: Some(elastic_id),
            elastic_pw: Some(elastic_pw),
            elastic_pool_cnt,
        }
    }
}
