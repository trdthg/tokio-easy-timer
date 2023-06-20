pub mod cache {

    use chrono::{DateTime, Utc};
    use lazy_static::lazy_static;
    use parking_lot::RwLock;

    lazy_static! {
        static ref CURRENT_TIMESTAMP: RwLock<DateTime<Utc>> = RwLock::new(chrono::Utc::now());
    }

    pub fn init() {
        // TODO!
        std::thread::spawn(move || {
            println!("{}", "start cache runing");
            loop {
                update_cached_timestamp();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
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

mod test {
    #[test]
    fn test() {
        assert!(1 == 2);
        let t = std::time::Instant::now();
        let mut n = 0;
        for i in 0..1000000 {
            let time = chrono::Local::now();
            n += time.timestamp();
        }
        dbg!("{}", t.elapsed().as_nanos());
    }
}
