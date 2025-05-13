use crate::common::*;

use crate::repository::es_repository::*;

#[async_trait]
pub trait IndexClearService {
    async fn delete_index_from_rule(&self, index_name_prefix: &str) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct IndexClearServicePub;


#[async_trait]
impl IndexClearService for IndexClearServicePub {
    
    async fn delete_index_from_rule(&self, index_name_prefix: &str) -> Result<(), anyhow::Error> {
        
        let es_conn: ElasticConnGuard = get_elastic_guard_conn().await?;

        //es_conn.get_search_query(es_query, index_name).await?;
        
        Ok(())
    }
}