use alloy::primitives::Address;
use sqlx::{Pool, Postgres};

pub const SCHEMA_BLOCKCHAIN: &str = "blockchain";
pub const SCHEMA_CONFIG: &str = "config";

pub type DbPool = Pool<Postgres>;

/*
This Address alias exists only for readability purposes and not ensures underlying Address contains only user account address
*/
pub type AccountAddress = Address;
pub type AccountAddressBin = [u8; 20];

pub fn get_placeholders_str(placeholders_amount: usize, rows_amount: usize) -> String {
    let mut ind = 0;
    let mut placeholders = Vec::<String>::with_capacity(rows_amount);

    for _ in 0..rows_amount {
        let mut x = Vec::<String>::with_capacity(placeholders_amount * 3);
        for _ in 0..placeholders_amount {
            ind += 1;
            x.push(format!("${}", ind));
        }
        let cur_placeholder = x.join(",");
        placeholders.push(format!("({})", cur_placeholder));
    }

    placeholders.join(",")
}

#[cfg(test)]
pub mod tests {
    use super::get_placeholders_str;

    #[test]
    pub fn test_1() {
        let res = get_placeholders_str(1, 1);
        assert_eq!("($1)", res);
    }

    #[test]
    pub fn test_2() {
        let res = get_placeholders_str(4, 1);
        assert_eq!("($1,$2,$3,$4)", res);
    }

    #[test]
    pub fn test_3() {
        let res = get_placeholders_str(5, 5);
        assert_eq!(
            "($1,$2,$3,$4,$5),($6,$7,$8,$9,$10),($11,$12,$13,$14,$15),($16,$17,$18,$19,$20),($21,$22,$23,$24,$25)",
            res
        );

        println!("{}", res);
    }
}
