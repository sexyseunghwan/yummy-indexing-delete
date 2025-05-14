use crate::common::*;

use crate::service::index_clear_service::*;

use crate::model::target_index::*;

use crate::utils_module::io_utils::*;

pub struct MainController<I: IndexClearService + Sync + Send + 'static> {
    index_clear_service: Arc<I>,
}

impl<I: IndexClearService + Sync + Send + 'static> MainController<I> {
    pub fn new(index_clear_service: Arc<I>) -> Self {
        Self {
            index_clear_service,
        }
    }

    #[doc = "메인 배치 함수"]
    pub async fn main_task(&self) -> Result<(), anyhow::Error> {
        /* 정리해줄 인덱스의 리스트를 뽑아준다. */
        let target_indexes: TargetIndexList = read_toml_file_from_env::<TargetIndexList>(
            "INDEX_LIST_PATH",
        )
        .expect("[Error][MainController->main_task] There was a problem reading 'target_indexes'.");

        /* 직렬처리 코드 */
        // for target_index in target_indexes.index {
        //     self.index_clear_service.delete_index_from_rule(&target_index).await?;
        // }

        /* 아래는 병렬처리 코드인데 크게 의미없는 듯.. 직렬처리해도 상관 없어보임 */
        let mut handles = vec![];

        for target_index in target_indexes.index {
            let service: Arc<I> = Arc::clone(&self.index_clear_service);

            let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> =
                tokio::spawn(async move { service.delete_index_from_rule(&target_index).await });

            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(inner_result) => {
                    if let Err(e) = inner_result {
                        error!("[Error][MainController->run_parallel] Stream task failed with error: {:?}", e);
                    }
                }
                Err(e) => {
                    error!(
                        "[Error][MainController->run_parallel] Tokio task join error: {:?}",
                        e
                    );
                }
            }
        }

        Ok(())
    }
}