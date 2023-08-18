use actix_web::{web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use ethers::providers::{Provider, Http};
use moka::sync::Cache;

mod handlers;
mod structs;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    // let bgl = "https//127.0.0.1:8332";
    let cache: Cache<String, f64> = Cache::new(10_000);
    let clonned_cache = cache.clone();
    tokio::spawn(async move {
        utils::get_owners_local(clonned_cache).await;
    });

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: r2d2::Pool<ConnectionManager<PgConnection>> = r2d2::Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");

    // let provider: Provider<Http> = Provider::<Http>::try_from("NO").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cache.clone()))
            // .app_data(web::Data::new(provider.clone()))
            .route("/", web::get().to(|| async { "Actix REST API" }))
            .service(handlers::get_nfts)
            .service(handlers::get_nft_by_address)
            .service(handlers::get_owners)
            .service(handlers::init_db)
            .service(handlers::get_wbgl)
            .service(handlers::get_last_trade)
            .service(handlers::get_pages)
            .service(handlers::get_payment)
            .service(handlers::get_blockchain_data)
            .service(handlers::get_last_winners)
            .service(handlers::get_lucky_hash)
            .service(handlers::get_tickets)
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
