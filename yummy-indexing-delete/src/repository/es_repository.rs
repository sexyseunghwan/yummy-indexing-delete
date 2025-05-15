use crate::common::*;

use crate::configs::elastic_server_config::*;

static ELASTICSEARCH_CONN_SEMAPHORE_POOL: once_lazy<Vec<Arc<EsRepositoryPub>>> = once_lazy::new(
    || {
        let config: ElasticServerConfig = ElasticServerConfig::new();

        let pool_cnt: i32 = *config.elastic_pool_cnt();
        let es_host: &Vec<String> = config.elastic_host();
        let es_id: String = config.elastic_id().clone().unwrap_or(String::from(""));
        let es_pw: String = config.elastic_pw().clone().unwrap_or(String::from(""));

        (0..pool_cnt)
        .map(|_| {
            Arc::new(
                EsRepositoryPub::new(es_host.clone(), &es_id, &es_pw)
                    .expect("[Error][ELASTICSEARCH_CONN_SEMA_POOL] Failed to create Elasticsearch client"),
            )
        })
        .collect()
    },
);

static SEMAPHORE: once_lazy<Arc<Semaphore>> = once_lazy::new(|| {
    let config: ElasticServerConfig = ElasticServerConfig::new();
    Arc::new(Semaphore::new(*config.elastic_pool_cnt() as usize))
});

#[derive(Debug)]
pub struct ElasticConnGuard {
    client: Arc<EsRepositoryPub>,
    _permit: OwnedSemaphorePermit, /* drop 시 자동 반환 */
}

impl ElasticConnGuard {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let permit: OwnedSemaphorePermit = SEMAPHORE.clone().acquire_owned().await?;

        /* 임의로 하나의 클라이언트를 가져옴 (랜덤 선택 가능) */
        let client: Arc<EsRepositoryPub> = ELASTICSEARCH_CONN_SEMAPHORE_POOL
            .choose(&mut rand::thread_rng())
            .cloned()
            .expect("[Error][EalsticConnGuard -> new] No clients available");

        Ok(Self {
            client,
            _permit: permit, /* Drop 시 자동 반환 */
        })
    }
}

impl Deref for ElasticConnGuard {
    type Target = EsRepositoryPub;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Drop for ElasticConnGuard {
    fn drop(&mut self) {
        info!("[ElasticConnGuard] permit dropped (semaphore released)");
    }
}

pub async fn get_elastic_guard_conn() -> Result<ElasticConnGuard, anyhow::Error> {
    info!("use elasticsearch connection");
    ElasticConnGuard::new().await
}

#[async_trait]
pub trait EsRepository {
    async fn get_index_belong_pattern(&self, index_pattern: &str) -> Result<Value, anyhow::Error>;
    async fn delete_index(&self, index_name: &str) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_clients: Vec<EsClient>,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch,
}

impl EsRepositoryPub {
    pub fn new(es_url_vec: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        let mut es_clients: Vec<EsClient> = Vec::new();

        for url in es_url_vec {
            let parse_url: String = format!("http://{}:{}@{}", es_id, es_pw, url);

            let es_url: Url = Url::parse(&parse_url)?;
            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: EsTransport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5, 0))
                .build()?;

            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: EsClient = EsClient::new(url, elastic_conn);

            es_clients.push(es_client);
        }

        Ok(EsRepositoryPub { es_clients })
    }

    #[doc = "Common logic: common node failure handling and node selection"]
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error: Option<anyhow::Error> = None;

        let mut rng: StdRng = StdRng::from_entropy();
        let mut shuffled_clients = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);

        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }
}

#[async_trait]
impl EsRepository for EsRepositoryPub {
    #[doc = "특정 인덱스 자체를 삭제해주는 함수."]
    /// # Arguments
    /// * `index_name` - 삭제할 인덱스 명
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn delete_index(&self, index_name: &str) -> Result<(), anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response = es_client
                    .es_conn
                    .indices()
                    .delete(IndicesDeleteParts::Index(&[index_name]))
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }

    #[doc = "특정 인덱스 패턴에 속하는 인덱스 전부를 가져와주는 함수."]
    /// # Arguments
    /// * `index_pattern` - 인덱스 패턴 문자열
    ///
    /// # Returns
    /// * Result<Value, anyhow::Error>
    async fn get_index_belong_pattern(&self, index_pattern: &str) -> Result<Value, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response = es_client
                    .es_conn
                    .cat()
                    .indices(CatIndicesParts::Index(&[index_pattern]))
                    .format("json")
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let response_body = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }
}
