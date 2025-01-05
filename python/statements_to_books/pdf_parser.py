from pathlib import Path
from pypdf import PdfReader


def page_count_of_pdf(pdf_file_path: Path | str) -> int:
    reader = PdfReader(pdf_file_path)
    return len(reader.pages)
