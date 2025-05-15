use crate::common::*;

use crate::repository::es_repository::*;

use crate::utils_module::time_utils::*;

use crate::model::target_index::*;

#[async_trait]
pub trait IndexClearService {
    async fn delete_index_from_rule(&self, target_index: &TargetIndex)
        -> Result<(), anyhow::Error>;
    fn parsed_data_index(&self, index_name: &str) -> Result<NaiveDate, anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct IndexClearServicePub;

#[async_trait]
impl IndexClearService for IndexClearServicePub {
    #[doc = "규칙에 의거하여 인덱스를 지워주는 함수"]
    /// # Arguments
    /// * `target_index` - 대상이 되는 인덱스 정보
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn delete_index_from_rule(
        &self,
        target_index: &TargetIndex,
    ) -> Result<(), anyhow::Error> {

        /* Elasitcsearch 커넥션 */
        let es_conn: ElasticConnGuard = get_elastic_guard_conn().await?;

        let res: Value = es_conn
            .get_index_belong_pattern(target_index.index_name())
            .await?;
        
        let cur_utc_time: NaiveDate = get_current_utc_naivedate();

        if let Some(index_list) = res.as_array() {
            for index in index_list {
                let index_name = index["index"].as_str()
                    .ok_or_else(|| anyhow!("[Error][IndexClearService->delete_index_from_rule] index['index'] variable not found."))?;

                let parsed_date: NaiveDate = match self.parsed_data_index(index_name) {
                    Ok(parsed_date) => parsed_date,
                    Err(e) => {
                        error!("{:?}", e);
                        continue;
                    }
                };
                
                /* 보존기한 데드라인 일자. */
                let perserve_days_ago: NaiveDate =
                    cur_utc_time - chrono::Duration::days(target_index.duration_days as i64);

                if parsed_date <= perserve_days_ago {
                    /* 인덱스 삭제 */
                    match es_conn.delete_index(index_name).await {
                        Ok(_) => {
                            info!("{} has been successfully deleted.", index_name);
                        }
                        Err(e) => {
                            error!("[Error][IndexClearService->delete_index_from_rule] {:?}", e);
                            continue;
                        }
                    }
                    info!("{}", index_name);
                }
            }
        }

        Ok(())
    }

    #[doc = "인덱스에 존재하는 날짜 양식을 날짜 포멧으로 뽑아주는 함수"]
    /// # Arguments
    /// * `index_name` - 대상이 되는 인덱스 정보
    ///
    /// # Returns
    /// * Result<NaiveDate, anyhow::Error>  
    fn parsed_data_index(&self, index_name: &str) -> Result<NaiveDate, anyhow::Error> {
        let regex: Regex = Regex::new(r"(\d{4}[-_]?\d{2}[-_]?\d{2})")?;

        /* 날짜 포멧 검증 및 수정 */
        let date_format: String = match regex
            .find(index_name)
            .map(|mat| mat.as_str().replace("_", "-"))
        {
            Some(date_format) => date_format,
            None => {
                return Err(anyhow!("[Error][IndexClearService->delete_index_from_rule] Error parsing variable 'date_format' into regular expression. : {}", index_name));
            }
        };

        /* 현재 인덱스의 생성일자 파악 */
        let parsed_date: NaiveDate = match NaiveDate::parse_from_str(&date_format, "%Y-%m-%d") {
            Ok(parsed_date) => parsed_date,
            Err(_e) => match NaiveDate::parse_from_str(&date_format, "%Y%m%d") {
                Ok(parsed_date) => parsed_date,
                Err(e) => {
                    return Err(anyhow!("[Parsing Error][delete_cluster_index()] An error occurred while converting 'parsed_date' data. // date_format: {:?}, {:?}", date_format, e));
                }
            },
        };

        Ok(parsed_date)
    }
}
