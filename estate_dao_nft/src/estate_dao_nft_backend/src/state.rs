use serde::Serialize;
use candid::{types::number::Nat, CandidType, Deserialize, Principal};
use std::collections::HashMap;
// use icrc_ledger_types::icrc1::account::Account;
 
pub type Subaccount = [u8; 32];
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];
 
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Account{ 
    pub owner: Principal,
    pub subaccount: Option<Subaccount>
}
 
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct RoyaltyData{
    pub royalty_share: u16,
    pub royalty_account: Account,
}
 
impl Default for Status {
    fn default() -> Self {
        Status::Draft
    }
}
 
// const default_status: Status = Status::default();
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum Status{
    Draft,
    Upcoming,
    Live,
    Ended,
}
 
//Collection level metadata
//Royalty structure is not fixed yet in ICRC7
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct CollectionMetadata {
    pub collection_id: String,
    pub name: String,
    pub desc: String,
    pub total_supply: u16,
    pub supply_cap: u16,
    pub property_images: Vec<String>,
    pub additional_metadata: Option<AdditionalMetadata>,
    pub status: Status,
    pub owner: String
}
 
//Additional metadata
//Royalty structure is not fixed yet in ICRC7
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct AdditionalMetadata {
    pub additional_details: Option<AdditionalDetails>,
    pub financial_details: Option<FinancialDetails>,
    pub documents: Vec<HashMap<String, String>>,
    pub market_details: Option<MarketDetails>,
}
 
//Documents metadata
//Royalty structure is not fixed yet in ICRC7
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct AdditionalDetails {
    pub last_renovation: Option<String>,
    pub beds: Option<u32>,
    pub year_built: Option<u32>, 
    pub square_footage: Option<f32>,
    pub flood_zone: Option<bool>,
    pub occupied: Option<bool>,
    pub baths: Option<u32>,
    pub monthly_rent: Option<f32>,
    pub crime_score: Option<u32>,
    pub school_score: Option<u32>,
    pub affordability: Option<f32>,
    pub price_per_sq_foot: Option<f32>,
}
 
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct InvestmentFinancials {
    pub underlying_asset_price: Option<f32>,
    pub platform_closing_fee: Option<f32>,
    pub initial_mainatance_reserve: Option<f32>, 
    pub min_investment: Option<f32>,     
}
 
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct ReturnsFinancials {
    pub total_5_year_irr: Option<f32>,
    pub projected_appreciation: Option<f32>,
    pub average_5_year_roi: Option<f32>,
    pub cap_rate: Option<f32>, 
    pub yields: Option<f32>,
}
 
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct RentFinancials {
    pub vacancy_rate: Option<f32>,
    pub monthly_utiliiies: Option<f32>,
    pub property_managment_fee: Option<f32>,
    pub llc_monthly_franchise_tax: Option<f32>,
    pub property_taxes: Option<f32>,
    pub projected_rent: Option<f32>,
}
 
//Documents metadata
//Royalty structure is not fixed yet in ICRC7
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct FinancialDetails {
    pub investment: Option<InvestmentFinancials>,
    pub returns: Option<ReturnsFinancials>,
    pub rents: Option<RentFinancials>,
    pub property_insurance: Option<f32>,
    pub expense_to_income_ratio: Option<f32>,
    pub total_monthly_cost: Option<f32>,
    pub monthly_cash_flow: Option<f32>,
}
 
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct MarketDetails {
    pub coordinates: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub median_home_sale_price: Option<u32>,
    pub average_rent: Option<u32>,
    pub annual_popullation_growth: Option<u32>,
    pub description: Option<String>,
}
 
// NFT specific data
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NFTMetadata {
    pub collection_id: String,
    pub nft_symbol: String,
    pub nft_token_id: String,
    pub nft_uri: String //image
}
 
// NFTMetadata + CollectionMetadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Metadata {
    pub collection_id: String,
    pub nft_symbol: String,
    pub nft_token_id: String,
    pub nft_uri: String, //image   
    pub collection_name: String,
    pub desc: String,
    // pub royalty: RoyaltyData,
    pub total_supply: u16,
    pub supply_cap: u16,
}