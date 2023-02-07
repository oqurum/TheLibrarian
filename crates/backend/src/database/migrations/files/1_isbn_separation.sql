-- Create ISBN Table
CREATE TABLE book_isbn (
    isbn	TEXT NOT NULL UNIQUE,
    book_id INT NOT NULL references book(id) ON DELETE CASCADE
);

-- Insert Pre-existing ISBN's into Table
INSERT INTO book_isbn SELECT isbn_13 as isbn, id as book_id FROM book WHERE isbn_13 IS NOT NULL;
INSERT INTO book_isbn SELECT isbn_10 as isbn, id as book_id FROM book WHERE isbn_10 IS NOT NULL;

-- Delete Both columns from Book Table
ALTER TABLE book DROP COLUMN isbn_10;
ALTER TABLE book DROP COLUMN isbn_13;