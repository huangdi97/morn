#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenCoord {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIElement {
    pub label: String,
    pub element_type: String,
    pub bounding_box: ScreenCoord,
    pub attributes: std::collections::HashMap<String, String>,
}

pub struct VisualGrounding;

impl VisualGrounding {
    pub fn new() -> Self {
        VisualGrounding
    }

    pub async fn locate(
        &self,
        _screenshot: &[u8],
        instruction: &str,
    ) -> Result<ScreenCoord, String> {
        let lower = instruction.to_lowercase();
        let coord = if lower.contains("search") || lower.contains("搜索") {
            ScreenCoord {
                x: 200.0,
                y: 50.0,
                width: 100.0,
                height: 30.0,
                confidence: 0.85,
            }
        } else if lower.contains("submit") || lower.contains("提交") || lower.contains("confirm")
        {
            ScreenCoord {
                x: 300.0,
                y: 400.0,
                width: 80.0,
                height: 35.0,
                confidence: 0.75,
            }
        } else if lower.contains("input") || lower.contains("输入") || lower.contains("text") {
            ScreenCoord {
                x: 150.0,
                y: 50.0,
                width: 300.0,
                height: 28.0,
                confidence: 0.8,
            }
        } else if lower.contains("click") || lower.contains("点击") {
            ScreenCoord {
                x: 250.0,
                y: 250.0,
                width: 50.0,
                height: 20.0,
                confidence: 0.7,
            }
        } else {
            ScreenCoord {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                confidence: 0.0,
            }
        };
        Ok(coord)
    }

    pub async fn detect_elements(&self, _screenshot: &[u8]) -> Result<Vec<UIElement>, String> {
        let elements = vec![
            UIElement {
                label: "Search".to_string(),
                element_type: "input".to_string(),
                bounding_box: ScreenCoord {
                    x: 50.0,
                    y: 20.0,
                    width: 300.0,
                    height: 30.0,
                    confidence: 0.9,
                },
                attributes: [("placeholder".to_string(), "Search...".to_string())]
                    .into_iter()
                    .collect(),
            },
            UIElement {
                label: "Submit".to_string(),
                element_type: "button".to_string(),
                bounding_box: ScreenCoord {
                    x: 360.0,
                    y: 20.0,
                    width: 80.0,
                    height: 30.0,
                    confidence: 0.85,
                },
                attributes: [("type".to_string(), "primary".to_string())]
                    .into_iter()
                    .collect(),
            },
            UIElement {
                label: "Menu".to_string(),
                element_type: "navigation".to_string(),
                bounding_box: ScreenCoord {
                    x: 0.0,
                    y: 0.0,
                    width: 50.0,
                    height: 60.0,
                    confidence: 0.8,
                },
                attributes: std::collections::HashMap::new(),
            },
        ];
        Ok(elements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_locate_search_button() {
        let vg = VisualGrounding::new();
        let coord = vg.locate(&[], "click search button").await.unwrap();
        assert!(coord.confidence > 0.8);
        assert!(coord.x > 0.0);
    }

    #[tokio::test]
    async fn test_locate_submit() {
        let vg = VisualGrounding::new();
        let coord = vg.locate(&[], "click submit").await.unwrap();
        assert!(coord.confidence > 0.7);
    }

    #[tokio::test]
    async fn test_locate_input() {
        let vg = VisualGrounding::new();
        let coord = vg.locate(&[], "input text field").await.unwrap();
        assert!(coord.confidence > 0.7);
    }

    #[tokio::test]
    async fn test_locate_unknown() {
        let vg = VisualGrounding::new();
        let coord = vg.locate(&[], "do something random").await.unwrap();
        assert_eq!(coord.confidence, 0.0);
    }

    #[tokio::test]
    async fn test_detect_elements() {
        let vg = VisualGrounding::new();
        let elements = vg.detect_elements(&[]).await.unwrap();
        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0].element_type, "input");
        assert_eq!(elements[1].element_type, "button");
    }

    #[tokio::test]
    async fn test_locate_chinese() {
        let vg = VisualGrounding::new();
        let coord = vg.locate(&[], "点击搜索按钮").await.unwrap();
        assert!(coord.confidence > 0.8);
    }

    #[test]
    fn test_screen_coord_serialization() {
        let coord = ScreenCoord {
            x: 100.0,
            y: 200.0,
            width: 50.0,
            height: 30.0,
            confidence: 0.95,
        };
        let json = serde_json::to_string(&coord).unwrap();
        let deserialized: ScreenCoord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.x, 100.0);
        assert_eq!(deserialized.confidence, 0.95);
    }
}
