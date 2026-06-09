use sdwah::router;
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
    let listener = TcpListener::bind("0.0.0.0:9527").await?;
    let app = router::init();
    axum::serve(listener, app).await?;
    Ok(())
}
