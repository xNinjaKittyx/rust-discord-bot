use std::sync::LazyLock;

pub static FOOTER_URL: LazyLock<String> = LazyLock::new(|| std::env::var("FOOTER_URL").unwrap());

pub static LOCALAI_URL: LazyLock<String> = LazyLock::new(|| std::env::var("LOCALAI_URL").unwrap());

pub static SERVE_STATIC_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("SERVE_STATIC_URL").unwrap());

pub static SHOKO_SERVER_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("SHOKO_SERVER_URL").unwrap());

pub static SHOKO_SERVER_API_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("SHOKO_SERVER_API_KEY").unwrap());

pub static SONARR_URL: LazyLock<String> = LazyLock::new(|| std::env::var("SONARR_URL").unwrap());

pub static SONARR_API_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("SONARR_API_KEY").unwrap());
