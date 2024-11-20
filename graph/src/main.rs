mod queries;
use eyre::Error;
use reqwest::Client;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // let http_client = Client::new();
    // let request_body = ActiveUsersQuery::build_query(queries::active_users_query::Variables {});

    // // Send the request to the Aave subgraph
    // let response = client
    //     .post(AAVE_SUBGRAPH_URL)
    //     .json(&request_body)
    //     .send()
    //     .await?
    //     .json::<Response<ResponseData>>()
    //     .await?;

    // // Process and print the response data
    // if let Some(data) = response.data {
    //     for user in data.users {
    //         println!(
    //             "User ID: {}, Collateral: {}, Debt: {}, Health Factor: {}",
    //             user.id, user.total_collateral_eth, user.total_debt_eth, user.health_factor,
    //         );
    //     }
    // } else if let Some(errors) = response.errors {
    //     eprintln!("GraphQL errors: {:?}", errors);
    // }

    Ok(())
}
