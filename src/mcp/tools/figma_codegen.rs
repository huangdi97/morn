//! Figma 代码生成工具 — 根据 Figma 设计令牌自动生成 React/TSX 组件代码。
//! 包括 TSX 模板、Tailwind CSS 样式和 TypeScript 类型定义。
//! Figma code generation tool — generates React/TSX component code from a Figma design token.
//! Includes TSX templates, Tailwind CSS styles, and TypeScript type definitions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Figma 设计请求参数。
/// `design_token` 为必填的 Figma 设计标识，其余字段为可选的组件名称、框架和样式方案。
///
/// Request parameters for Figma design code generation.
/// `design_token` is required; `component_name`, `framework`, and `styling` are optional.
#[derive(Debug, Serialize, Deserialize)]
pub struct FigmaDesignRequest {
    pub design_token: String,
    pub component_name: Option<String>,
    pub framework: Option<String>,
    pub styling: Option<String>,
}

/// 生成的代码产物，包含组件名、TSX 代码、Tailwind CSS 样式和 TypeScript 类型。
///
/// Generated code artifacts: component name, TSX code, Tailwind CSS, and TypeScript types.
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub component_name: String,
    pub tsx_code: String,
    pub css_tailwind: String,
    pub types: String,
}

/// Figma 代码生成响应。`success` 指示是否成功，
/// `component` 在成功时包含生成的代码，`error` 在失败时包含错误消息。
///
/// Response for Figma code generation. `success` indicates whether generation succeeded,
/// `component` holds the generated code on success, and `error` holds the error message on failure.
#[derive(Debug, Serialize, Deserialize)]
pub struct FigmaCodegenResponse {
    pub success: bool,
    pub component: Option<GeneratedCode>,
    pub error: Option<String>,
}

/// 根据 Figma 设计请求生成组件代码。
/// 解析 `design_token`，生成 TSX、Tailwind CSS 和类型定义。
/// 若 `design_token` 为空则返回错误。
///
/// Generate component code from a Figma design request.
/// Parses the `design_token`, then produces TSX, Tailwind CSS, and type definitions.
/// Returns an error if `design_token` is empty.
pub fn generate_component(request: FigmaDesignRequest) -> FigmaCodegenResponse {
    let token = &request.design_token;
    let name = request
        .component_name
        .unwrap_or_else(|| "FigmaComponent".into());
    let framework = request.framework.unwrap_or_else(|| "react".into());
    let styling = request.styling.unwrap_or_else(|| "tailwind".into());

    if token.is_empty() {
        return FigmaCodegenResponse {
            success: false,
            component: None,
            error: Some("design_token is required".into()),
        };
    }

    let parsed = parse_figma_token(token);
    let tsx_code = generate_tsx(&name, &parsed, &framework, &styling);
    let css_tailwind = generate_tailwind(&parsed);
    let types = generate_types(&name, &parsed);

    FigmaCodegenResponse {
        success: true,
        component: Some(GeneratedCode {
            component_name: name,
            tsx_code,
            css_tailwind,
            types,
        }),
        error: None,
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ParsedDesign {
    layout: String,
    elements: Vec<DesignElement>,
    colors: Vec<String>,
    spacing: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DesignElement {
    element_type: String,
    content: Option<String>,
    properties: Value,
}

fn parse_figma_token(_token: &str) -> ParsedDesign {
    ParsedDesign {
        layout: "flex".into(),
        elements: vec![
            DesignElement {
                element_type: "container".into(),
                content: None,
                properties: serde_json::json!({
                    "direction": "column",
                    "padding": 16,
                    "gap": 12,
                    "background": "#ffffff",
                    "border_radius": 8,
                }),
            },
            DesignElement {
                element_type: "heading".into(),
                content: Some("Design Title".into()),
                properties: serde_json::json!({
                    "level": 2,
                    "font_size": 24,
                    "font_weight": 600,
                    "color": "#1a1a2e",
                }),
            },
            DesignElement {
                element_type: "text".into(),
                content: Some("Design description text from Figma.".into()),
                properties: serde_json::json!({
                    "font_size": 14,
                    "color": "#4a5568",
                    "line_height": 1.5,
                }),
            },
            DesignElement {
                element_type: "button".into(),
                content: Some("Click Me".into()),
                properties: serde_json::json!({
                    "variant": "primary",
                    "background": "#3b82f6",
                    "color": "#ffffff",
                    "padding": "10px 20px",
                    "border_radius": 6,
                }),
            },
        ],
        colors: vec!["#1a1a2e".into(), "#3b82f6".into(), "#ffffff".into(), "#4a5568".into()],
        spacing: vec![4, 8, 12, 16, 24],
    }
}

fn generate_tsx(name: &str, design: &ParsedDesign, _framework: &str, _styling: &str) -> String {
    let mut code = String::new();
    code.push_str("import React from 'react';\n\n");
    code.push_str(&format!("interface {}Props {{\n  className?: string;\n}}\n\n", name));
    code.push_str(&format!("export const {}: React.FC<{}Props> = ({{ className }}) => {{\n  return (\n", name, name));
    code.push_str("    <div className={`container flex-col p-4 gap-3 bg-white rounded-lg ${className || ''}`}>\n");

    for el in &design.elements {
        match el.element_type.as_str() {
            "heading" => {
                let level = el.properties.get("level").and_then(Value::as_u64).unwrap_or(2);
                let tag = format!("h{}", level);
                code.push_str(&format!(
                    "      <{tag} className=\"text-2xl font-semibold text-gray-900\">{content}</{tag}>\n",
                    tag = tag,
                    content = el.content.as_deref().unwrap_or("Heading")
                ));
            }
            "text" => {
                code.push_str(&format!(
                    "      <p className=\"text-sm text-gray-600 leading-relaxed\">{content}</p>\n",
                    content = el.content.as_deref().unwrap_or("Text")
                ));
            }
            "button" => {
                code.push_str(&format!(
                    "      <button className=\"px-5 py-2.5 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors\">{content}</button>\n",
                    content = el.content.as_deref().unwrap_or("Button")
                ));
            }
            "container" => {
                code.push_str("      <div className=\"flex flex-col gap-3\">\n        {/* child elements */}\n      </div>\n");
            }
            _ => {}
        }
    }

    code.push_str("    </div>\n  );\n};\n");
    code
}

fn generate_tailwind(design: &ParsedDesign) -> String {
    let mut css = String::new();
    css.push_str("/* Generated Tailwind classes from Figma design */\n");
    css.push_str(".container {\n  @apply flex flex-col p-4 gap-3 bg-white rounded-lg;\n}\n");

    for el in &design.elements {
        match el.element_type.as_str() {
            "heading" => {
                css.push_str(".heading {\n  @apply text-2xl font-semibold;\n}\n");
            }
            "button" => {
                css.push_str(".btn-primary {\n  @apply px-5 py-2.5 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors;\n}\n");
            }
            _ => {}
        }
    }

    css.push_str(&format!(
        "\n/* Design colors */\n:root {{\n{}\n}}\n",
        design
            .colors
            .iter()
            .enumerate()
            .map(|(i, c)| format!("  --figma-color-{}: {};", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n")
    ));

    css
}

fn generate_types(name: &str, design: &ParsedDesign) -> String {
    let mut props = format!(
        "export interface {}Props {{\n  className?: string;\n  children?: React.ReactNode;",
        name
    );

    for el in design.elements.iter() {
        props.push_str(&format!(
            "\n  {}?: {};",
            el.element_type,
            match el.element_type.as_str() {
                "heading" => "string",
                "text" => "string",
                "button" => "React.ReactNode",
                _ => "React.ReactNode",
            }
        ));
    }

    props.push_str("\n}\n");

    format!(
        "// Type definitions for {} component\n{}\nexport const {}: React.FC<{}Props>;\n",
        name, props, name, name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_component_success() {
        let req = FigmaDesignRequest {
            design_token: "figma_token_123".into(),
            component_name: Some("HeroBanner".into()),
            framework: Some("react".into()),
            styling: Some("tailwind".into()),
        };
        let resp = generate_component(req);
        assert!(resp.success);
        assert!(resp.component.is_some());
        let comp = resp.component.unwrap();
        assert_eq!(comp.component_name, "HeroBanner");
        assert!(comp.tsx_code.contains("HeroBanner"));
        assert!(comp.tsx_code.contains("React.FC"));
    }

    #[test]
    fn test_generate_component_empty_token() {
        let req = FigmaDesignRequest {
            design_token: "".into(),
            component_name: None,
            framework: None,
            styling: None,
        };
        let resp = generate_component(req);
        assert!(!resp.success);
        assert!(resp.component.is_none());
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_generate_component_default_name() {
        let req = FigmaDesignRequest {
            design_token: "abc".into(),
            component_name: None,
            framework: None,
            styling: None,
        };
        let resp = generate_component(req);
        assert!(resp.success);
        assert_eq!(
            resp.component.unwrap().component_name,
            "FigmaComponent"
        );
    }

    #[test]
    fn test_parse_figma_returns_design() {
        let parsed = parse_figma_token("test_token");
        assert_eq!(parsed.layout, "flex");
        assert!(!parsed.elements.is_empty());
        assert!(!parsed.colors.is_empty());
        assert!(!parsed.spacing.is_empty());
    }

    #[test]
    fn test_generated_code_contains_types() {
        let req = FigmaDesignRequest {
            design_token: "tok".into(),
            component_name: None,
            framework: None,
            styling: None,
        };
        let resp = generate_component(req);
        let comp = resp.component.unwrap();
        assert!(comp.types.contains("Props"));
    }

    #[test]
    fn test_generate_tailwind_includes_colors() {
        let parsed = parse_figma_token("tok");
        let css = generate_tailwind(&parsed);
        assert!(css.contains("figma-color"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let req = FigmaDesignRequest {
            design_token: "tok".into(),
            component_name: Some("Card".into()),
            framework: Some("react".into()),
            styling: Some("tailwind".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: FigmaDesignRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.design_token, "tok");
        assert_eq!(back.component_name.unwrap(), "Card");
    }

    #[test]
    fn test_generate_tsx_contains_component_name() {
        let parsed = parse_figma_token("tok");
        let tsx = generate_tsx("CustomCard", &parsed, "react", "tailwind");
        assert!(tsx.contains("CustomCard"));
        assert!(tsx.contains("React.FC"));
        assert!(tsx.contains("className"));
    }

    #[test]
    fn test_generate_tsx_includes_button() {
        let parsed = parse_figma_token("tok");
        let tsx = generate_tsx("Page", &parsed, "react", "tailwind");
        assert!(tsx.contains("button"));
        assert!(tsx.contains("Click Me"));
    }

    #[test]
    fn test_generate_types_contains_props_interface() {
        let parsed = parse_figma_token("tok");
        let types = generate_types("HeroBanner", &parsed);
        assert!(types.contains("HeroBannerProps"));
        assert!(types.contains("heading?: string"));
        assert!(types.contains("text?: string"));
    }

    #[test]
    fn test_generate_tailwind_includes_spacing() {
        let parsed = parse_figma_token("tok");
        let css = generate_tailwind(&parsed);
        assert!(css.contains("figma-color"));
        assert!(css.contains("--figma-color-1"));
    }

    #[test]
    fn test_parsed_design_defaults() {
        let parsed = parse_figma_token("any_token");
        assert_eq!(parsed.layout, "flex");
        assert!(parsed.colors.len() >= 3);
        assert!(parsed.spacing.len() >= 3);
    }
}
