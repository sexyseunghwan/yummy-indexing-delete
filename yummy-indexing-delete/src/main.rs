/*
Author      : Seunghwan Shin
Create date : 2025-05-14
Description :

History     : 2024-05-14 Seunghwan Shin       # [v.1.0.0] Yummy-project 에서 사용되는 로그를 주기적으로 정리해주는 프로그램.
*/
mod common;
use common::*;

mod utils_module;
use utils_module::logger_utils::*;

mod repository;

mod service;
use service::index_clear_service::*;

mod controller;
use controller::main_controller::*;

mod model;

mod configs;

#[tokio::main]
async fn main() {
    dotenv().ok();

    /* 전역 로거설정 */
    set_global_logger();

    let start = tokio::time::Instant::now();

    info!("Program Start");

    let index_clear_service: IndexClearServicePub = IndexClearServicePub::new();
    let main_controller: MainController<IndexClearServicePub> =
        MainController::new(Arc::new(index_clear_service));

    match main_controller.main_task().await {
        Ok(_) => (),
        Err(e) => {
            error!("{:?}", e);
        }
    }
    
    //let duration = start.elapsed(); // 경과 시간 측정
    // println!("⏱ 실행 시간: {:.3?}", duration);
}
// 테스트 코드
// for i in 0..10 {
//     let handle = tokio::spawn(async move {
//         let conn: ElasticConnGuard = ElasticConnGuard::new().await.unwrap();
//         let since = start_time.elapsed().as_secs_f32();
//         info!(
//             "[{:.3}s] Task {:02} acquired conn id : {}✅",
//             since, i, conn.client_id()
//         );

//         sleep(Duration::from_secs(3)).await;

//         let since = start_time.elapsed().as_secs_f32();
//         info!(
//             "[{:.3}s] Task {:02} released conn id {} ⛔",
//             since, i, conn.client_id()
//         );
//     });

//     handles.push(handle);
// }

// for h in handles {
//     let _ = h.await;
// }
