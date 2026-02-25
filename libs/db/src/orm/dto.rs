use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct ReqIdDto {
    pub id: i64,
}

#[derive(Serialize, Debug)]
pub struct ReqPaginationDto {
    pub page: Option<u64>,
    pub items_per_page: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct ResultPagination {
    pub current_page: u64,
    pub items_per_page: u64,
    pub total_items: u64,
    pub total_pages: u64,
}

#[derive(Serialize, Debug)]
pub struct ResFilterResultDto<T>
where
    T: Serialize,
{
    pub pagination: ResultPagination,
    pub items: Option<Vec<T>>,
}
