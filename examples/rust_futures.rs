use isahc::AsyncReadResponseExt;
use tokio::runtime::Runtime;
use rust_futures::{init, Http};

fn main() {
    let mut executor = init();
    executor.block_on(isahc_main());
}

// Use raw request
pub async fn async_main() {
    println!("Program starting");
    let txt = Http::get("/6000/HelloAsyncAwait1-1").await;
    println!("{txt}");
    let txt = Http::get("/4000/HelloAsyncAwait1-1").await;
    println!("{txt}");
}

// Use `reqwest` crate
pub async fn reqwest_main() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    println!("Program starting");
    let url = "http://127.0.0.1:8080/6000/HelloAsyncAwait1-1";
    let res = reqwest::get(url).await.unwrap();
    let txt = res.text().await.unwrap();
    println!("{txt}");
    let url = "http://127.0.0.1:8080/4000/HelloAsyncAwait1-1";
    let res = reqwest::get(url).await.unwrap();
    let txt = res.text().await.unwrap();
    println!("{txt}");
}

// Use `isahc` crate
pub async fn isahc_main() {
    println!("Program starting");
    let url = "http://127.0.0.1:8080/6000/HelloAsyncAwait1-1";
    let mut res = isahc::get_async(url).await.unwrap();
    let txt = res.text().await.unwrap();
    println!("{txt}");
    let url = "http://127.0.0.1:8080/4000/HelloAsyncAwait1-1";
    let mut res = isahc::get_async(url).await.unwrap();
    let txt = res.text().await.unwrap();
    println!("{txt}");
}
