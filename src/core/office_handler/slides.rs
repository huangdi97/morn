//! slides — Generates and manipulates presentation slide decks.
use super::slides_helper::*;
use super::{CacheEntry, OfficeHandler, SlideTemplate};
use std::io::Write;
use std::path::Path;

impl OfficeHandler {
    pub fn create_slide_from_template(
        &self,
        template: &SlideTemplate,
        title: &str,
        body: &str,
    ) -> Result<Vec<u8>, String> {
        let cache_key = format!("slide_{}_{}", template.id, uuid::Uuid::new_v4());
        let content_type = single_slide_content_types();
        let rels = rels_xml();
        let slide_xml = slide(title, body);
        let presentation_xml = single_slide_presentation();
        let presentation_rels = single_slide_presentation_rels();
        let slide_master_xml = slide_master();
        let slide_layout_xml = slide_layout();
        let theme_xml = theme();

        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
            let opts: zip::write::SimpleFileOptions = Default::default();
            let file_opts = opts;
            zip.add_directory("_rels/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/_rels/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slides/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slides/_rels/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideMasters/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideMasters/_rels/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideLayouts/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideLayouts/_rels/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/theme/", file_opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("docProps/", file_opts)
                .map_err(|e| e.to_string())?;

            zip.start_file("[Content_Types].xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(content_type.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("_rels/.rels", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(rels.as_bytes()).map_err(|e| e.to_string())?;

            zip.start_file("ppt/presentation.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(presentation_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/_rels/presentation.xml.rels", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(presentation_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/slides/slide1.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/slideMasters/slideMaster1.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_master_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let master_rels = slide_master_rels();
            zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(master_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/slideLayouts/slideLayout1.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_layout_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let layout_rels = slide_layout_rels();
            zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(layout_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/theme/theme1.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(theme_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let core_xml = core_properties("2025-01-01");
            zip.start_file("docProps/core.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(core_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let app_xml = app_properties(1);
            zip.start_file("docProps/app.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(app_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.finish().map_err(|e| e.to_string())?;
        }

        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(
            cache_key,
            CacheEntry {
                data: buffer.clone(),
                created_at: chrono::Utc::now().timestamp_millis(),
            },
        );

        Ok(buffer)
    }

    pub fn export_to_pptx(
        &self,
        slides: &[(String, String)],
        output_path: &str,
    ) -> Result<Vec<u8>, String> {
        let template = SlideTemplate {
            id: "default".into(),
            title: String::new(),
            body: String::new(),
        };

        if slides.is_empty() {
            return Err("At least one slide is required".into());
        }

        let slide_data = self.create_slide_from_template(&template, &slides[0].0, &slides[0].1)?;

        if slides.len() == 1 {
            let path = Path::new(output_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(output_path, &slide_data).map_err(|e| e.to_string())?;

            let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
            cache.insert(
                format!("pptx_{}", uuid::Uuid::new_v4()),
                CacheEntry {
                    data: slide_data.clone(),
                    created_at: chrono::Utc::now().timestamp_millis(),
                },
            );

            return Ok(slide_data);
        }

        let mut combined = Vec::new();
        let content_types = multi_slide_content_types(slides.len());
        let presentation_rels = presentation_rels(slides.len());
        let presentation_xml = presentation(slides.len());

        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut combined));
            let opts: zip::write::SimpleFileOptions = Default::default();
            zip.add_directory("_rels/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/", opts).map_err(|e| e.to_string())?;
            zip.add_directory("ppt/_rels/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slides/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slides/_rels/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideMasters/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideMasters/_rels/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideLayouts/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/slideLayouts/_rels/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("ppt/theme/", opts)
                .map_err(|e| e.to_string())?;
            zip.add_directory("docProps/", opts)
                .map_err(|e| e.to_string())?;

            zip.start_file("[Content_Types].xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(content_types.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("_rels/.rels", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(rels_xml().as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/presentation.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(presentation_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/_rels/presentation.xml.rels", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(presentation_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            for (i, (title, body)) in slides.iter().enumerate() {
                let slide_xml = slide(title, body);

                zip.start_file(format!("ppt/slides/slide{}.xml", i + 1), opts)
                    .map_err(|e| e.to_string())?;
                zip.write_all(slide_xml.as_bytes())
                    .map_err(|e| e.to_string())?;
            }

            zip.start_file("ppt/slideMasters/slideMaster1.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_master().as_bytes())
                .map_err(|e| e.to_string())?;

            let master_rels = slide_master_rels();
            zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(master_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/slideLayouts/slideLayout1.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_layout().as_bytes())
                .map_err(|e| e.to_string())?;

            let layout_rels = slide_layout_rels();
            zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(layout_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/theme/theme1.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(theme().as_bytes())
                .map_err(|e| e.to_string())?;

            let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let core_xml = core_properties(&date);
            zip.start_file("docProps/core.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(core_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let app_xml = app_properties(slides.len());
            zip.start_file("docProps/app.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(app_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.finish().map_err(|e| e.to_string())?;
        }

        let path = Path::new(output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(output_path, &combined).map_err(|e| e.to_string())?;

        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(
            format!("pptx_{}", uuid::Uuid::new_v4()),
            CacheEntry {
                data: combined.clone(),
                created_at: chrono::Utc::now().timestamp_millis(),
            },
        );

        Ok(combined)
    }
}
