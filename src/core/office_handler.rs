use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
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

impl OfficeHandler {
    pub fn new() -> Self {
        OfficeHandler {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_slide_from_template(
        &self,
        template: &SlideTemplate,
        title: &str,
        body: &str,
    ) -> Result<Vec<u8>, String> {
        let cache_key = format!("slide_{}_{}", template.id, uuid::Uuid::new_v4());

        let content_type = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
<Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
<Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
</Types>"#;

        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;

        let escaped_title = title
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\'', "&apos;")
            .replace('"', "&quot;");
        let escaped_body = body
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\'', "&apos;")
            .replace('"', "&quot;");

        let slide_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld>
<p:spTree>
<p:nvGrpSpPr><p:nvGrpSpPrPr/><p:nvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr/>
<p:sp>
<p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="457200"/><a:ext cx="8229600" cy="1371600"/></a:xfrm><a:prstGeom prst="rect"/></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="4400" b="1"/><a:t>{}</a:t></a:r></a:p></p:txBody>
</p:sp>
<p:sp>
<p:nvSpPr><p:cNvPr id="3" name="Body"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="1828800"/><a:ext cx="8229600" cy="5029200"/></a:xfrm><a:prstGeom prst="rect"/></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="2800"/><a:t>{}</a:t></a:r></a:p></p:txBody>
</p:sp>
</p:spTree>
</p:cSld>
<p:sldPr/>
</p:sld>"#,
            escaped_title, escaped_body
        );

        let presentation_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
<p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst>
<p:sldSz cx="9144000" cy="6858000"/>
<p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#;

        let presentation_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>"#;

        let slide_master_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree><p:nvGrpSpPr><p:nvGrpSpPrPr/><p:nvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
<p:sldMasterPr/>
</p:sldMaster>"#;

        let slide_layout_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree><p:nvGrpSpPr><p:nvGrpSpPrPr/><p:nvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
<p:sldLayoutPr/>
</p:sldLayout>"#;

        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Default">
<a:themeElements>
<a:clrScheme name="Default"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="44546A"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme>
<a:fontScheme name="Default"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>
<a:fmtScheme name="Default"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"/></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"/></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"/></a:gs></a:gsLst></a:gradFill></a:fillStyleLst><a:lnStyleLst><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"/></a:gs><a:gs pos="40000"><a:schemeClr val="phClr"/></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"/></a:gs></a:gsLst></a:gradFill></a:bgFillStyleLst></a:fmtScheme>
</a:themeElements>
</a:theme>"#;

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

            let master_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;
            zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(master_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/slideLayouts/slideLayout1.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_layout_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let layout_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#;
            zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(layout_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/theme/theme1.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(theme_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let core_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/">
<dc:creator>Morn OfficeHandler</dc:creator>
<cp:lastModifiedBy>Morn OfficeHandler</cp:lastModifiedBy>
<dcterms:created xsi:type="dcterms:W3CDTF">2025-01-01T00:00:00Z</dcterms:created>
<dcterms:modified xsi:type="dcterms:W3CDTF">2025-01-01T00:00:00Z</dcterms:modified>
</cp:coreProperties>"#;
            zip.start_file("docProps/core.xml", file_opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(core_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let app_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
<Application>Morn OfficeHandler</Application>
<SlideCount>1</SlideCount>
<TotalTime>0</TotalTime>
</Properties>"#;
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

        let content_type = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
<Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>"#;

        let mut content_types = content_type.to_string();
        for i in 0..slides.len() {
            content_types.push_str(&format!(
                r#"<Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#,
                i + 1
            ));
        }
        content_types.push_str("\n</Types>");

        let mut presentation_rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>"#,
        );

        for i in 0..slides.len() {
            presentation_rels.push_str(&format!(
                r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#,
                i + 2,
                i + 1
            ));
        }
        presentation_rels.push_str("\n</Relationships>");

        let mut slide_ids = String::new();
        let offset: u32 = 256;
        for i in 0..slides.len() {
            slide_ids.push_str(&format!(
                r#"<p:sldId id="{}" r:id="rId{}"/>"#,
                offset + i as u32,
                i + 2
            ));
        }

        let presentation_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
<p:sldIdLst>{}</p:sldIdLst>
<p:sldSz cx="9144000" cy="6858000"/>
<p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
            slide_ids
        );

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
                let escaped_title = title
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('\'', "&apos;")
                    .replace('"', "&quot;");
                let escaped_body = body
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('\'', "&apos;")
                    .replace('"', "&quot;");

                let slide_xml = format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld>
<p:spTree>
<p:nvGrpSpPr><p:nvGrpSpPrPr/><p:nvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr/>
<p:sp>
<p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="457200"/><a:ext cx="8229600" cy="1371600"/></a:xfrm><a:prstGeom prst="rect"/></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="4400" b="1"/><a:t>{}</a:t></a:r></a:p></p:txBody>
</p:sp>
<p:sp>
<p:nvSpPr><p:cNvPr id="3" name="Body"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="1828800"/><a:ext cx="8229600" cy="5029200"/></a:xfrm><a:prstGeom prst="rect"/></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="2800"/><a:t>{}</a:t></a:r></a:p></p:txBody>
</p:sp>
</p:spTree>
</p:cSld>
<p:sldPr/>
</p:sld>"#,
                    escaped_title, escaped_body
                );

                zip.start_file(&format!("ppt/slides/slide{}.xml", i + 1), opts)
                    .map_err(|e| e.to_string())?;
                zip.write_all(slide_xml.as_bytes())
                    .map_err(|e| e.to_string())?;
            }

            zip.start_file("ppt/slideMasters/slideMaster1.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_master().as_bytes())
                .map_err(|e| e.to_string())?;

            let master_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;
            zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(master_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/slideLayouts/slideLayout1.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(slide_layout().as_bytes())
                .map_err(|e| e.to_string())?;

            let layout_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#;
            zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(layout_rels.as_bytes())
                .map_err(|e| e.to_string())?;

            zip.start_file("ppt/theme/theme1.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(theme().as_bytes())
                .map_err(|e| e.to_string())?;

            let core_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/">
<dc:creator>Morn OfficeHandler</dc:creator>
<cp:lastModifiedBy>Morn OfficeHandler</cp:lastModifiedBy>
<dcterms:created xsi:type="dcterms:W3CDTF">{}T00:00:00Z</dcterms:created>
<dcterms:modified xsi:type="dcterms:W3CDTF">{}T00:00:00Z</dcterms:modified>
</cp:coreProperties>"#,
                chrono::Utc::now().format("%Y-%m-%d"),
                chrono::Utc::now().format("%Y-%m-%d")
            );
            zip.start_file("docProps/core.xml", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(core_xml.as_bytes())
                .map_err(|e| e.to_string())?;

            let app_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
<Application>Morn OfficeHandler</Application>
<SlideCount>{}</SlideCount>
<TotalTime>0</TotalTime>
</Properties>"#,
                slides.len()
            );
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

    pub fn export_to_csv(
        &self,
        data: &[Vec<String>],
        output_path: &str,
    ) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(std::io::Cursor::new(&mut buffer));
            for row in data {
                writer.write_record(row).map_err(|e| e.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
        }

        let path = Path::new(output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(output_path, &buffer).map_err(|e| e.to_string())?;

        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(
            format!("csv_{}", uuid::Uuid::new_v4()),
            CacheEntry {
                data: buffer.clone(),
                created_at: chrono::Utc::now().timestamp_millis(),
            },
        );

        Ok(buffer)
    }

    pub fn export_to_xlsx(
        &self,
        sheet_name: &str,
        headers: &[&str],
        rows: &[Vec<&str>],
        output_path: &str,
    ) -> Result<Vec<u8>, String> {
        use rust_xlsxwriter::*;

        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        sheet.set_name(sheet_name).map_err(|e| e.to_string())?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        for (col, header) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        let cell_format = Format::new();
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                sheet
                    .write_string_with_format(
                        (row_idx + 1) as u32,
                        col_idx as u16,
                        *value,
                        &cell_format,
                    )
                    .map_err(|e| e.to_string())?;
            }
        }

        let buffer = workbook.save_to_buffer().map_err(|e| e.to_string())?;

        let path = Path::new(output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(output_path, &buffer).map_err(|e| e.to_string())?;

        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(
            format!("xlsx_{}", uuid::Uuid::new_v4()),
            CacheEntry {
                data: buffer.clone(),
                created_at: chrono::Utc::now().timestamp_millis(),
            },
        );

        Ok(buffer)
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

fn rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#.to_string()
}

fn slide_master() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree><p:nvGrpSpPr><p:nvGrpSpPrPr/><p:nvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
<p:sldMasterPr/>
</p:sldMaster>"#.to_string()
}

fn slide_layout() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree><p:nvGrpSpPr><p:nvGrpSpPrPr/><p:nvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
<p:sldLayoutPr/>
</p:sldLayout>"#.to_string()
}

fn theme() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Default">
<a:themeElements>
<a:clrScheme name="Default"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="44546A"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme>
<a:fontScheme name="Default"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>
<a:fmtScheme name="Default"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"/></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"/></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"/></a:gs></a:gsLst></a:gradFill></a:fillStyleLst><a:lnStyleLst><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"/></a:gs><a:gs pos="40000"><a:schemeClr val="phClr"/></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"/></a:gs></a:gsLst></a:gradFill></a:bgFillStyleLst></a:fmtScheme>
</a:themeElements>
</a:theme>"#.to_string()
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
