use database::{Lookup, Pool};
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    env_logger::init();

    let sqlite = std::env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("needs sqlite database as 1st argument"));
    let postgres = std::env::args()
        .nth(2)
        .unwrap_or_else(|| panic!("needs postgres database as 2nd argument"));

    let sqlite = Pool::open(&sqlite);
    let mut sqlite_conn = sqlite.connection().await;
    let postgres = Pool::open(&postgres);
    let postgres_conn = postgres.connection().await;

    let sqlite_idx = sqlite_conn.load_index().await;

    let cid_name = format!("imported-{}", chrono::Utc::now().timestamp());
    println!("Collection ID for import is {}", cid_name);
    let cid = postgres_conn.collection_id(&cid_name).await;

    let mut benchmarks = HashSet::new();

    // Starting after the sqlite and postgres db args, the rest are artifact
    // names to import.
    for artifact in std::env::args().skip(3) {
        let aid = sqlite_conn
            .artifact_by_name(&artifact)
            .await
            .unwrap_or_else(|| {
                panic!("{} not found in sqlite db", artifact);
            });

        let sqlite_aid = sqlite_conn.artifact_id(&aid).await;
        let postgres_aid = postgres_conn.artifact_id(&aid).await;

        for &(krate, profile, cache, stat) in sqlite_idx.all_pstat_series() {
            if benchmarks.insert(krate) {
                postgres_conn.record_benchmark(krate.as_str(), None).await;
            }

            let id = database::DbLabel::ProcessStat {
                krate,
                profile,
                cache,
                stat,
            }
            .lookup(&sqlite_idx)
            .unwrap();

            let value = sqlite_conn
                .get_pstats(&[id], &[Some(sqlite_aid)])
                .await
                .pop()
                .unwrap()
                .pop()
                .unwrap();
            if let Some(value) = value {
                postgres_conn
                    .record_statistic(
                        cid,
                        postgres_aid,
                        &krate.to_string(),
                        profile,
                        cache,
                        stat.as_str(),
                        value,
                    )
                    .await;
            }
        }
    }
}
