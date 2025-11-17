use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Standard pagination query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// Number of items per page (max 100)
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    
    /// Sort field (optional)
    pub sort_by: Option<String>,
    
    /// Sort direction: "asc" or "desc"
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

fn default_sort_order() -> SortOrder {
    SortOrder::Desc
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Desc
    }
}

impl PaginationParams {
    /// Validate and normalize pagination parameters
    pub fn validate(&mut self) -> Result<(), String> {
        // Ensure page is at least 1
        if self.page < 1 {
            self.page = 1;
        }
        
        // Limit page size to 100
        if self.page_size < 1 {
            self.page_size = default_page_size();
        } else if self.page_size > 100 {
            self.page_size = 100;
        }
        
        Ok(())
    }
    
    /// Calculate SQL LIMIT value
    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }
    
    /// Calculate SQL OFFSET value
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }
    
    /// Get sort direction as SQL string
    pub fn sort_direction(&self) -> &str {
        match self.sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

/// Pagination metadata for responses
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub current_page: u32,
    
    /// Total number of pages
    pub total_pages: u32,
    
    /// Total number of items across all pages
    pub total_items: i64,
    
    /// Number of items per page
    pub items_per_page: u32,
    
    /// Whether there is a next page
    pub has_next: bool,
    
    /// Whether there is a previous page
    pub has_previous: bool,
}

impl PaginationMeta {
    /// Create pagination metadata from query params and total count
    pub fn new(params: &PaginationParams, total_items: i64) -> Self {
        let total_pages = if total_items == 0 {
            1
        } else {
            ((total_items as f64) / (params.page_size as f64)).ceil() as u32
        };
        
        Self {
            current_page: params.page,
            total_pages,
            total_items,
            items_per_page: params.page_size,
            has_next: params.page < total_pages,
            has_previous: params.page > 1,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The data items for the current page
    pub data: Vec<T>,
    
    /// Pagination metadata
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: i64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(params, total_items),
        }
    }
}

/// Filter parameters for list endpoints
#[derive(Debug, Deserialize, IntoParams)]
pub struct FilterParams {
    /// Filter by status
    pub status: Option<String>,
    
    /// Filter by date from (ISO 8601)
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter by date to (ISO 8601)
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Search query (text search)
    pub search: Option<String>,
}

impl FilterParams {
    /// Check if any filters are active
    pub fn has_filters(&self) -> bool {
        self.status.is_some() 
            || self.from_date.is_some() 
            || self.to_date.is_some() 
            || self.search.is_some()
    }
}

/// Combined pagination and filter parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQueryParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// Number of items per page (max 100)
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    
    /// Sort field (optional)
    pub sort_by: Option<String>,
    
    /// Sort direction: "asc" or "desc"
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
    
    /// Filter by status
    pub status: Option<String>,
    
    /// Filter by date from (ISO 8601)
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter by date to (ISO 8601)
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Search query (text search)
    pub search: Option<String>,
}

impl ListQueryParams {
    /// Get pagination parameters
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            page_size: self.page_size,
            sort_by: self.sort_by.clone(),
            sort_order: self.sort_order,
        }
    }
    
    /// Get filter parameters
    pub fn filters(&self) -> FilterParams {
        FilterParams {
            status: self.status.clone(),
            from_date: self.from_date,
            to_date: self.to_date,
            search: self.search.clone(),
        }
    }
    
    /// Validate and normalize parameters
    pub fn validate(&mut self) -> Result<(), String> {
        // Validate pagination
        if self.page < 1 {
            self.page = 1;
        }
        
        if self.page_size < 1 {
            self.page_size = default_page_size();
        } else if self.page_size > 100 {
            self.page_size = 100;
        }
        
        // Validate date range
        if let (Some(from), Some(to)) = (self.from_date, self.to_date) {
            if from > to {
                return Err("from_date must be before to_date".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Calculate SQL LIMIT value
    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }
    
    /// Calculate SQL OFFSET value
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }
    
    /// Get sort direction as SQL string
    pub fn sort_direction(&self) -> &str {
        match self.sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_defaults() {
        let params = PaginationParams {
            page: default_page(),
            page_size: default_page_size(),
            sort_by: None,
            sort_order: default_sort_order(),
        };
        
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
        assert_eq!(params.offset(), 0);
        assert_eq!(params.limit(), 20);
    }
    
    #[test]
    fn test_pagination_params_offset() {
        let params = PaginationParams {
            page: 3,
            page_size: 10,
            sort_by: None,
            sort_order: SortOrder::Desc,
        };
        
        assert_eq!(params.offset(), 20); // (3-1) * 10
        assert_eq!(params.limit(), 10);
    }
    
    #[test]
    fn test_pagination_params_validation() {
        let mut params = PaginationParams {
            page: 0,
            page_size: 200,
            sort_by: None,
            sort_order: SortOrder::Asc,
        };
        
        params.validate().unwrap();
        
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 100); // Capped at 100
    }
    
    #[test]
    fn test_pagination_meta() {
        let params = PaginationParams {
            page: 2,
            page_size: 10,
            sort_by: None,
            sort_order: SortOrder::Desc,
        };
        
        let meta = PaginationMeta::new(&params, 45);
        
        assert_eq!(meta.current_page, 2);
        assert_eq!(meta.total_pages, 5);
        assert_eq!(meta.total_items, 45);
        assert_eq!(meta.items_per_page, 10);
        assert!(meta.has_next);
        assert!(meta.has_previous);
    }
    
    #[test]
    fn test_paginated_response() {
        let params = PaginationParams {
            page: 1,
            page_size: 10,
            sort_by: None,
            sort_order: SortOrder::Desc,
        };
        
        let data = vec![1, 2, 3, 4, 5];
        let response = PaginatedResponse::new(data, &params, 50);
        
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.pagination.total_items, 50);
        assert_eq!(response.pagination.total_pages, 5);
    }
}
