from pathlib import Path
from pypdf import PdfReader


def extract_text_from_pdfs_to_txt_files(input_dir: str, output_dir: str) -> str:
    file_ext = 'pdf'
    pdf_files = get_filepath_or_filepaths(input_dir, file_ext)
    if len(pdf_files) == 1:
        pass
    else:
        raise NotImplementedError('Handling of multiple files is not implemented.')

    pdf_file_path = pdf_files
    raw_text_name = Path(pdf_file_path).name.replace(".pdf", "")
    name = f"{raw_text_name}_extracted_text.txt"
    text_file_path = Path(output_dir, name)
    pdf_file_str = ""
    page_sep = "--page--"
    num_pages_in_pdf = page_count_of_pdf(pdf_file_path)
    print(f"Pages in pdf: {num_pages_in_pdf}")
    for page_num in range(num_pages_in_pdf):
        page_text_str = extract_text_from_pdf(pdf_file_path, page=page_num)
        if page_num == 0:
            pdf_file_str = page_text_str
        else:
            pdf_file_str += f"{page_sep}{page_text_str}"
    if not text_file_path.exists():
        with open(text_file_path, 'w') as f:
            f.write(page_text_str)

    return pdf_file_str


def get_filepath_or_filepaths(dir: Path | str, ext: str) -> list[None | Path]:
    p = Path(dir)
    files = p.glob(f"**/*.{ext}")
    return list(files)


def page_count_of_pdf(pdf_file_path: Path | str):
    reader = PdfReader(pdf_file_path)
    return len(reader.pages)


def extract_text_from_pdf(pdf_file_path, page=0):
    print('Process single file')
    reader = PdfReader(pdf_file_path)
    page = reader.pages[page]
    page_text = page.extract_text()
    return page_text


if __name__ == "__main__":
    input_dir = '.'
    file_ext = 'pdf'
    pdf_files = get_filepath_or_filepaths(input_dir, file_ext)
    if len(pdf_files) == 1:
        pass
    else:
        raise NotImplementedError('Handling of multiple files is not implemented.')

    for fp in pdf_files:
        num_pages_in_pdf = page_count_of_pdf(fp)
        print(f"Pages in pdf: {num_pages_in_pdf}")
        page_text_str = extract_text_from_pdf(fp)
        raw_name = Path(fp).name.replace(".pdf", "")
        name = f"{raw_name}_extracted_text.txt"
        with open(Path(input_dir, name), 'w') as f:
            f.write(page_text_str)
