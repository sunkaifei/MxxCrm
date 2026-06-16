//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


#[cfg(feature = "enable-es")]
use elasticsearch::Elasticsearch;
#[cfg(feature = "enable-es")]
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
#[cfg(feature = "enable-es")]
use elasticsearch::http::Url;

#[cfg(feature = "enable-es")]
pub static ES_CLIENT: LazyLock<Elasticsearch> = LazyLock::new(|| {
    // 创建 Elasticsearch 实例
    //let transport = Transport::single_node("http://127.0.0.1:9200").unwrap();
    //let es_client = Elasticsearch::new(transport);

    let url = Url::parse("http://127.0.0.1:9200").unwrap();
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool).disable_proxy().build().unwrap();
    let es_client = Elasticsearch::new(transport);
    
    es_client
});

// lazy_static! {
//     static ref SCHEDULER: Scheduler<Local> = {
//         let (scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
//         scheduler
//     };
// 
//     static ref SCHED_SERVICE: LocalSet<Local> = {
//         let (scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
//         sched_service
//     };
// }