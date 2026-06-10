use sdwah::{ch03_developing_endpoints, router};
use tokio::net::TcpListener;

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name(env!("CARGO_PKG_NAME"))
        .enable_all()
        .build()?;
    rt.block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite:todos.db")
        .await?;
    let state = ch03_developing_endpoints::Ch03State::new_arc(pool);
    let listener = TcpListener::bind("0.0.0.0:9527").await?;
    let app = router::init(state);
    axum::serve(listener, app).await?;
    Ok(())
}
