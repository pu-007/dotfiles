import os
import re
from pypdf import PdfReader, PdfWriter


def sanitize_filename(filename):
    """Removes characters that are invalid for file names."""
    return re.sub(r'[\\/*?:"<>|]', "", filename).strip()


def split_pdf_by_toc(pdf_path):
    """
    Splits a PDF file into multiple files based on its top-level table of contents.
    """
    try:
        # 1. Set up paths
        base_dir = os.path.dirname(pdf_path)
        output_dir = os.path.join(base_dir, "鬼吹灯_split")
        os.makedirs(output_dir, exist_ok=True)
        print(f"Output directory: '{output_dir}'")

        reader = PdfReader(pdf_path)
        total_pages = len(reader.pages)
        outlines = reader.outline

        if not outlines:
            print("Error: No table of contents (bookmarks) found in the PDF.")
            return

        # 2. Extract chapter info (title and start page) from top-level outlines
        chapters = []
        for item in outlines:
            # We only process the top-level items (Dest objects)
            # Nested items are lists, which we ignore for this logic.
            if hasattr(item, 'title') and hasattr(item, 'page'):
                try:
                    page_num = reader.get_page_number(item.page)
                    chapters.append({
                        "title": item.title,
                        "start_page": page_num
                    })
                except Exception:
                    # Sometimes a bookmark might be invalid, skip it.
                    print(
                        f"Warning: Could not resolve page for bookmark '{item.title}'. Skipping."
                    )
                    continue

        if not chapters:
            print(
                "Error: Could not find any valid top-level chapters in the table of contents."
            )
            return

        print(f"Found {len(chapters)} top-level chapters to split.")

        # 3. Determine page ranges and create a new PDF for each chapter
        for i, chapter in enumerate(chapters):
            title = sanitize_filename(chapter["title"])
            start_page = chapter["start_page"]

            # Determine the end page
            if i + 1 < len(chapters):
                # The end page is the page before the next chapter starts
                end_page = chapters[i + 1]["start_page"] - 1
            else:
                # This is the last chapter, so it goes to the end of the document
                end_page = total_pages - 1

            if start_page > end_page:
                print(
                    f"Warning: Chapter '{title}' seems to have no pages (starts at {start_page+1}, ends at {end_page+1}). Skipping."
                )
                continue

            print(
                f"  -> Processing '{title}' (pages {start_page + 1}-{end_page + 1})..."
            )

            # 4. Create the new PDF writer and add pages
            writer = PdfWriter()
            for page_num in range(start_page, end_page + 1):
                writer.add_page(reader.pages[page_num])

            # 5. Save the new PDF
            output_filename = os.path.join(output_dir, f"{title}.pdf")
            with open(output_filename, "wb") as output_pdf:
                writer.write(output_pdf)

            print(f"     Saved to '{output_filename}'")

        print("\nPDF splitting complete!")

    except FileNotFoundError:
        print(f"Error: The file was not found at '{pdf_path}'")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")


if __name__ == "__main__":
    # The PDF file to be split
    input_pdf = "/mnt/d/Downloads/鬼吹灯（合集）.pdf"
    split_pdf_by_toc(input_pdf)
