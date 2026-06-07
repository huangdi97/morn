use super::{CacheEntry, OfficeHandler};
use std::path::Path;

impl OfficeHandler {
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
}
