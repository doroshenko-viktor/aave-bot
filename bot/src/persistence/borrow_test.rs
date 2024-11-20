#[cfg(test)]
mod tests {
    use sqlx::types::time::{OffsetDateTime, PrimitiveDateTime};

    #[test]
    fn test_timestamp_convert() {
        let timestamp = 4295032668;
        match OffsetDateTime::from_unix_timestamp(timestamp) {
            Ok(x) => {
                let x = PrimitiveDateTime::new(x.date(), x.time());
                println!("PrimitiveDateTime: {}", x);
            }
            Err(e) => eprintln!("Can't parse timestamp: {}", e),
        }
        // .map(|x| )
        // .ok()
    }
}
