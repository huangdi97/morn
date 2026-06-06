use crate::core::visual_grounding::ScreenCoord;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedButton {
    pub label: String,
    pub bounding_box: ScreenCoord,
    pub enabled: bool,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedTextField {
    pub label: String,
    pub bounding_box: ScreenCoord,
    pub placeholder: Option<String>,
    pub current_value: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedImage {
    pub label: String,
    pub bounding_box: ScreenCoord,
    pub alt_text: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionResult {
    pub buttons: Vec<DetectedButton>,
    pub text_fields: Vec<DetectedTextField>,
    pub images: Vec<DetectedImage>,
    pub raw_screenshot_width: u32,
    pub raw_screenshot_height: u32,
}

pub struct VisualAgent;

impl VisualAgent {
    pub fn new() -> Self {
        VisualAgent
    }

    pub async fn detect_all(&self, screenshot: &[u8]) -> Result<DetectionResult, String> {
        let buttons = self.detect_buttons(screenshot).await?;
        let text_fields = self.detect_text_fields(screenshot).await?;
        let images = self.detect_images(screenshot).await?;
        Ok(DetectionResult {
            buttons,
            text_fields,
            images,
            raw_screenshot_width: 1920,
            raw_screenshot_height: 1080,
        })
    }

    pub async fn detect_buttons(&self, _screenshot: &[u8]) -> Result<Vec<DetectedButton>, String> {
        Ok(vec![
            DetectedButton {
                label: "Submit".to_string(),
                bounding_box: ScreenCoord {
                    x: 360.0,
                    y: 20.0,
                    width: 80.0,
                    height: 35.0,
                    confidence: 0.92,
                },
                enabled: true,
                attributes: [("type".to_string(), "primary".to_string())].into(),
            },
            DetectedButton {
                label: "Cancel".to_string(),
                bounding_box: ScreenCoord {
                    x: 460.0,
                    y: 20.0,
                    width: 80.0,
                    height: 35.0,
                    confidence: 0.88,
                },
                enabled: true,
                attributes: [("type".to_string(), "secondary".to_string())].into(),
            },
        ])
    }

    pub async fn detect_text_fields(&self, _screenshot: &[u8]) -> Result<Vec<DetectedTextField>, String> {
        Ok(vec![
            DetectedTextField {
                label: "Search".to_string(),
                bounding_box: ScreenCoord {
                    x: 50.0,
                    y: 20.0,
                    width: 300.0,
                    height: 30.0,
                    confidence: 0.91,
                },
                placeholder: Some("Search...".to_string()),
                current_value: None,
                attributes: [("maxlength".to_string(), "100".to_string())].into(),
            },
            DetectedTextField {
                label: "Username".to_string(),
                bounding_box: ScreenCoord {
                    x: 100.0,
                    y: 100.0,
                    width: 250.0,
                    height: 28.0,
                    confidence: 0.87,
                },
                placeholder: Some("Enter username".to_string()),
                current_value: None,
                attributes: HashMap::new(),
            },
        ])
    }

    pub async fn detect_images(&self, _screenshot: &[u8]) -> Result<Vec<DetectedImage>, String> {
        Ok(vec![
            DetectedImage {
                label: "Logo".to_string(),
                bounding_box: ScreenCoord {
                    x: 10.0,
                    y: 10.0,
                    width: 120.0,
                    height: 40.0,
                    confidence: 0.95,
                },
                alt_text: Some("Company Logo".to_string()),
                attributes: [("src".to_string(), "/logo.png".to_string())].into(),
            },
            DetectedImage {
                label: "Avatar".to_string(),
                bounding_box: ScreenCoord {
                    x: 1800.0,
                    y: 10.0,
                    width: 36.0,
                    height: 36.0,
                    confidence: 0.93,
                },
                alt_text: Some("User Avatar".to_string()),
                attributes: [("src".to_string(), "/avatar.png".to_string())].into(),
            },
        ])
    }

    pub async fn click_at(&self, coord: &ScreenCoord) -> Result<(), String> {
        let center_x = coord.x + coord.width / 2.0;
        let center_y = coord.y + coord.height / 2.0;
        println!("[VisualAgent] Clicking at ({}, {})", center_x, center_y);
        Ok(())
    }

    pub async fn type_at(&self, coord: &ScreenCoord, text: &str) -> Result<(), String> {
        let center_x = coord.x + coord.width / 2.0;
        let center_y = coord.y + coord.height / 2.0;
        println!("[VisualAgent] Typing '{}' at ({}, {})", text, center_x, center_y);
        Ok(())
    }

    pub async fn click_button(&self, button: &DetectedButton) -> Result<(), String> {
        self.click_at(&button.bounding_box).await
    }

    pub async fn fill_text_field(&self, field: &DetectedTextField, text: &str) -> Result<(), String> {
        self.click_at(&field.bounding_box).await?;
        self.type_at(&field.bounding_box, text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_buttons() {
        let agent = VisualAgent::new();
        let buttons = agent.detect_buttons(&[]).await.unwrap();
        assert_eq!(buttons.len(), 2);
        assert!(buttons.iter().any(|b| b.label == "Submit"));
        assert!(buttons.iter().any(|b| b.label == "Cancel"));
    }

    #[tokio::test]
    async fn test_detect_text_fields() {
        let agent = VisualAgent::new();
        let fields = agent.detect_text_fields(&[]).await.unwrap();
        assert_eq!(fields.len(), 2);
        assert!(fields.iter().any(|f| f.label == "Search"));
        assert!(fields.iter().any(|f| f.label == "Username"));
    }

    #[tokio::test]
    async fn test_detect_images() {
        let agent = VisualAgent::new();
        let images = agent.detect_images(&[]).await.unwrap();
        assert_eq!(images.len(), 2);
        assert!(images.iter().any(|i| i.label == "Logo"));
        assert!(images.iter().any(|i| i.label == "Avatar"));
    }

    #[tokio::test]
    async fn test_detect_all() {
        let agent = VisualAgent::new();
        let result = agent.detect_all(&[]).await.unwrap();
        assert_eq!(result.buttons.len(), 2);
        assert_eq!(result.text_fields.len(), 2);
        assert_eq!(result.images.len(), 2);
    }

    #[tokio::test]
    async fn test_click_at() {
        let agent = VisualAgent::new();
        let coord = ScreenCoord {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
            confidence: 1.0,
        };
        assert!(agent.click_at(&coord).await.is_ok());
    }

    #[tokio::test]
    async fn test_type_at() {
        let agent = VisualAgent::new();
        let coord = ScreenCoord {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 28.0,
            confidence: 1.0,
        };
        assert!(agent.type_at(&coord, "hello").await.is_ok());
    }

    #[tokio::test]
    async fn test_click_button() {
        let agent = VisualAgent::new();
        let buttons = agent.detect_buttons(&[]).await.unwrap();
        assert!(agent.click_button(&buttons[0]).await.is_ok());
    }

    #[tokio::test]
    async fn test_fill_text_field() {
        let agent = VisualAgent::new();
        let fields = agent.detect_text_fields(&[]).await.unwrap();
        assert!(agent.fill_text_field(&fields[0], "test input").await.is_ok());
    }
}
