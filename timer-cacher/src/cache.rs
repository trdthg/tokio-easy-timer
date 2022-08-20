pub mod cache {

    use std::{collections::HashMap, sync::Arc};

    use chrono::{DateTime, TimeZone, Utc};
    use lazy_static::lazy_static;
    use parking_lot::{Mutex, RwLock};
    use timer_utils::extensions::Extensions;

    lazy_static! {
        static ref CURRENT_TIMESTAMP: RwLock<DateTime<Utc>> = RwLock::new(chrono::Utc::now());
        static ref extensions: Extensions = Extensions::default();
    }

    pub fn init() {
        // TODO!
        std::thread::spawn(move || loop {
            update_cached_timestamp();
            std::thread::sleep(std::time::Duration::from_secs(1));
        });
    }

    pub fn get_cached_datetime() -> DateTime<Utc> {
        *CURRENT_TIMESTAMP.read()
    }

    pub fn get_cached_timestamp() -> u64 {
        CURRENT_TIMESTAMP.read().timestamp() as u64
    }

    pub fn update_cached_timestamp() {
        *CURRENT_TIMESTAMP.write() = chrono::Utc::now();
    }
}
