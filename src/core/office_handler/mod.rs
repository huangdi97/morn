use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SlideTemplate {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub created_at: i64,
}

pub struct OfficeHandler {
    cache: Mutex<HashMap<String, CacheEntry>>,
}

impl Default for OfficeHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub mod documents;
pub mod slides;
pub mod spreadsheets;

impl OfficeHandler {
    pub fn new() -> Self {
        OfficeHandler {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_cached(&self, key: &str) -> Option<CacheEntry> {
        self.cache.lock().ok()?.get(key).cloned()
    }

    pub fn clear_cache(&self) -> Result<usize, String> {
        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        let count = cache.len();
        cache.clear();
        Ok(count)
    }

    pub fn cache_size(&self) -> Result<usize, String> {
        let cache = self.cache.lock().map_err(|e| e.to_string())?;
        Ok(cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_slide_from_template() {
        let handler = OfficeHandler::new();
        let template = SlideTemplate {
            id: "test".into(),
            title: "Test Template".into(),
            body: "Template body".into(),
        };
        let result = handler.create_slide_from_template(&template, "Hello", "World");
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_export_to_pptx_single_slide() {
        let handler = OfficeHandler::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.pptx");
        let result =
            handler.export_to_pptx(&[("Title".into(), "Body".into())], path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_export_to_pptx_empty_slides() {
        let handler = OfficeHandler::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.pptx");
        let result = handler.export_to_pptx(&[], path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_export_to_csv() {
        let handler = OfficeHandler::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.csv");
        let data = vec![
            vec!["name".into(), "age".into()],
            vec!["Alice".into(), "30".into()],
            vec!["Bob".into(), "25".into()],
        ];
        let result = handler.export_to_csv(&data, path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Alice"));
    }

    #[test]
    fn test_export_to_xlsx() {
        let handler = OfficeHandler::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.xlsx");
        let result = handler.export_to_xlsx(
            "Sheet1",
            &["Name", "Age"],
            &[vec!["Alice", "30"], vec!["Bob", "25"]],
            path.to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_cache_operations() {
        let handler = OfficeHandler::new();
        assert_eq!(handler.cache_size().unwrap(), 0);

        let template = SlideTemplate {
            id: "cache-test".into(),
            title: String::new(),
            body: String::new(),
        };
        handler
            .create_slide_from_template(&template, "Cached", "Slide")
            .unwrap();
        assert_eq!(handler.cache_size().unwrap(), 1);

        let count = handler.clear_cache().unwrap();
        assert_eq!(count, 1);
        assert_eq!(handler.cache_size().unwrap(), 0);
    }
}
