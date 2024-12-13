use sqlx::query_as;

use super::{
    borrow::BorrowEventDto, liquidation::LiquidationEventDto, repay::RepayEventDto,
    supply::SupplyEventDto, withdraw::WithdrawEventDto,
};

// pub enum Event {
//     Supply(SupplyEventDto),
//     Borrow(BorrowEventDto),
//     Repay(RepayEventDto),
//     Withdraw(WithdrawEventDto),
//     Liquidation(LiquidationEventDto),
// }
// pub async fn get_sorted_events(pool: &PostgresPool, from_block: i32, to_block: i32) -> Vec<Event> {
//     query!(r#"
//         select 

        
//     "#);
// }
