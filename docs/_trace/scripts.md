# Trace — bộ công cụ script (spec → vị trí hiện thực)

Tài liệu nội bộ cho người bảo trì spec.

| Spec area | Vị trí hiện thực |
|---|---|
| SCR-EXPORT-GSHEET, SCAN-R1..R8 | `analyze_tag.py` `analyze()` |
| SCR-EXPORT-GSHEET-LITE | `action/tags_reader.py` `TagsReader.readToGoogleSheet()` |
| SCR-EXPORT-EXCEL | `analyze_tag_excel_v2.py` `analyze()`, `lib/excel_client.py` `ExcelSheetWriter` |
| SCAN-R2 (Google), SCAN-R8 | `lib/gsheet_client.py` `GoogleSheetWriter.__init__` (`add_worksheet`) |
| SCAN-R5 | `lib/utils.py` `countFiles()` |
| TAGDATA-R1..R7 | thư viện `music_tag`, các vòng `os.walk` ở các file trên |
| SCR-EDIT-SET, EDIT-R1, R2 | `set_tag.py` `setTags()` |
| SCR-EDIT-REPLACE, EDIT-R3..R6 | `replace_tag.py` `checkAndReplaceTagName()` |
| SCR-SYNC-APPLY, SYNC-R1..R5 | `update_fix_tag.py` `update()` dòng 11–62 |
| SCR-SYNC-MOVE, SYNC-R6 | `update_fix_tag.py` dòng 64–83 |
| SCR-CLEAN-CUEAPE, CLEAN-R1..R5 | `tools/remove_full_file.py` `run()` |
| SCR-UI-DESKTOP | `gui.py` (PyQt6) |
| SCR-UI-WEB | `main.py` (Flask) |
| SCR-UI-SIMPLE | `app.py` (PySimpleGUI) |
| SCR-UTIL-CRAWL | `crawl_web.py` |
| SCR-UTIL-FSHARE | `fshare_download.py`, `lib/fshare_client.py` |
