use crate::MornError;
use crate::AppState;
use tauri::State;

use morn::market::{Listing, Marketplace, Review};

#[tauri::command]
pub(crate) fn search_market_listings(
    query: Option<String>,
    type_filter: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let storage = state.storage.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());

    let listings = match type_filter.as_deref() {
        Some(t) if !t.is_empty() && t != "all" => marketplace.list(Some(t)),
        _ => marketplace.list(None),
    };

    let filtered = match query.as_deref() {
        Some(q) if !q.is_empty() => {
            let q_lower = q.to_lowercase();
            listings
                .into_iter()
                .filter(|l| {
                    l.name.to_lowercase().contains(&q_lower)
                        || l.description.to_lowercase().contains(&q_lower)
                })
                .collect::<Vec<Listing>>()
        }
        _ => listings,
    };

    serde_json::to_value(filtered).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn submit_review(
    listing_id: String,
    rating: u8,
    comment: String,
    state: State<AppState>,
) -> Result<String, MornError> {
    if rating < 1 || rating > 5 {
        return Err(MornError::Internal("Rating must be between 1 and 5".to_string()));
    }

    let storage = state.storage.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let marketplace = Marketplace::new(s.clone());
    marketplace
        .get(&listing_id)
        .ok_or_else(|| "Listing not found".to_string())?;

    let review = Review {
        id: format!("rev-{}", uuid::Uuid::new_v4()),
        listing_id: listing_id.clone(),
        user_id: "anonymous".to_string(),
        rating,
        comment,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    s.save_review(&review)?;

    let listing = marketplace.get(&listing_id).ok_or_else(|| MornError::Internal("Listing not found after submit".to_string()))?;
    let reviews = s.get_listing_reviews(&listing_id)?;
    let avg: f64 = if reviews.is_empty() {
        rating as f64
    } else {
        reviews.iter().map(|r| r.rating as f64).sum::<f64>() / reviews.len() as f64
    };
    s.update_listing_rating(&listing_id, avg, listing.downloads)?;

    Ok(review.id)
}

#[tauri::command]
pub(crate) fn get_listing_reviews(
    listing_id: String,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let storage = state.storage.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let reviews = s.get_listing_reviews(&listing_id)?;
    serde_json::to_value(reviews).map_err(|e| MornError::Internal(e.to_string()))
}
