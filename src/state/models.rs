use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum RaffleStatus {
    Created,
    Active,
    Drawing,
    Canceled,
    Completed,
}
