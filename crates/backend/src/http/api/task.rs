use futures::TryStreamExt;
use tokio_postgres::Client;

use crate::Result;




pub fn run_task() {
    //
}



// JOIN - Includes ALL, even if book_person is NULL

// SELECT DISTINCT ON(id) id, cached, book_person.person_id
// FROM book
// LEFT OUTER JOIN book_person ON book_person.book_id = book.id
// ORDER BY book.id ASC


// JOIN - Includes only the ones which match book_person

// SELECT DISTINCT ON(id) id, cached, book_person.person_id
// FROM book
// JOIN book_person ON book_person.book_id = book.id
// ORDER BY book.id ASC

// SELECT DISTINCT ON(book.id) book.id, book.cached, person.id, person.name
// FROM book
// JOIN book_person ON book_person.book_id = book.id
// JOIN person ON person.id = book_person.person_id
// ORDER BY book.id ASC



// NON JOIN - Select ALL which has a person

// SELECT DISTINCT ON(id) id, cached
// FROM book
// WHERE id IN (SELECT book_id FROM book_person)
// ORDER BY book.id ASC


// NON JOIN - Select ALL which does NOT have a person

// SELECT DISTINCT ON(id) id, cached
// FROM book
// WHERE id NOT IN (SELECT book_id FROM book_person)
// ORDER BY book.id ASC


pub async fn refresh_book_cache(db: &Client) -> Result<()> {
    let values = db.query(
        r#"
            SELECT DISTINCT ON(book.id) book.id, cached, book_person.person_id, person.name
            FROM book
            JOIN book_person ON book_person.book_id = book.id
            JOIN person ON person.id = book_person.person_id
            ORDER BY book.id ASC
        "#,
        &[]
    ).await?;

    for row in values {
        let book_id: i32 = row.get(0);
        let book_cache = common_local::MetadataItemCached::from_string(&row.get::<_, String>(1));
        let person_id: i32 = row.get(2);
        let person_name: String = row.get(3);

        db.execute(
            "UPDATE book SET cached = $2 WHERE id = $1",
            params![
                book_id,
                book_cache
                    .author(person_name)
                    .author_id(common::PersonId::from(person_id as usize))
                    .as_string()
            ]
        ).await?;
    }

    Ok(())
}


fn slice_iter<'a>(
    s: &'a [&'a (dyn tokio_postgres::types::ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn tokio_postgres::types::ToSql> + 'a {
    s.iter().map(|s| *s as _)
}