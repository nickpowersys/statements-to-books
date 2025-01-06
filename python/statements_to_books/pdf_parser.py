from pathlib import Path
from pypdf import PdfReader


def page_count_of_pdf(pdf_file_path: Path | str) -> int:
    reader = PdfReader(pdf_file_path)
    return len(reader.pages)


def extract_text_from_pdf(pdf_file_path: Path, page: int) -> str:
    print('Process single file')
    reader = PdfReader(pdf_file_path)
    page = reader.pages[page]
    page_text = page.extract_text()
    return page_text
