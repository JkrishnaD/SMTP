use std::env;

use once_cell::sync::Lazy;

pub struct Config {
    pub db_url: String,
}

// global config for env variables
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv::dotenv().ok();
    Config {
        db_url: env::var("DATABASE_URL").expect("DATABASE_URl not found"),
    }
});
