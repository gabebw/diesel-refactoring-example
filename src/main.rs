#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod models;
mod schema;

use crate::schema::posts;
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::SqliteConnection;
use diesel_migrations::embed_migrations;
use std::error::Error;

embed_migrations!("migrations");
diesel_infix_operator!(Glob, " GLOB ");

// Not the safest, since it claims to take a possibly-NULL text column and
// changes it to text, but works if you use it right. I'd love to make it safer,
// for example always adding a `NOT NULL` constraint whenever it's used.
sql_function! {
    #[sql_name = "lower"]
    fn lower_nullable(a: sql_types::Nullable<sql_types::Text>) -> sql_types::Text;
}

fn glob<T, U>(left: T, right: U) -> Glob<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    Glob::new(left, right.as_expression())
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

fn connect() -> Result<SqliteConnection, Box<dyn Error>> {
    let db = "./sqlite.db";
    let connection =
        SqliteConnection::establish(&db).unwrap_or_else(|_| panic!("Error connecting to {}", db));
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())?;
    Ok(connection)
}

/*
// This is very broken.

use diesel::dsl;

// , Glob<sql_types::Nullable<sql_types::Text>, U::Expression>>,
fn ilike<'a, T>(column_name: T, value: &str) -> dsl::Filter<posts::table, dsl::IsNotNull<T>>
where
    T: diesel::Expression,
    T::SqlType: sql_types::SingleValue,
{
    column_name.is_not_null()
    // .and(glob(lower_nullable(column_name), format!("*{}*", value)))
}
*/

pub fn search(name: Option<String>, body: Option<String>) -> Result<Vec<i32>, Box<dyn Error>> {
    let conn = connect()?;
    let mut query = posts::table.select(posts::id).into_boxed();

    // WHERE title IS NOT NULL AND lower(title) = GLOB('*something*')
    if let Some(n) = name {
        // A very rough `ILIKE` for SQlite
        query = query
            .filter(posts::title.is_not_null())
            .filter(glob(lower_nullable(posts::title), format!("*{}*", n)));
    }

    // WHERE body IS NOT NULL AND lower(body) = GLOB('*something*')
    if let Some(b) = body {
        /*
        How can I refactor this (and the block above) into something like:

            query = query.my_custom_ilike(posts::body, b)

        or even:

            query = my_custom_ilike(query, posts::body, b)

        */
        query = query
            .filter(posts::body.is_not_null())
            .filter(glob(lower_nullable(posts::body), format!("*{}*", b)));
    }

    Ok(query.load(&conn).map_err(Box::new)?)
}

fn main() {
    let new_posts = vec![
        NewPost {
            title: "first post",
            body: "this is the body for the first post",
        },
        NewPost {
            title: "Back at it again with a post",
            body: "another body",
        },
        NewPost {
            title: "This time is the charm",
            body: "third body",
        },
    ];
    let conn = connect().expect("could not connect");
    diesel::delete(posts::table)
        .execute(&conn)
        .expect("Error deleting posts");
    for new_post in new_posts {
        diesel::insert_into(posts::table)
            .values(&new_post)
            .execute(&conn)
            .expect("Error saving post");
    }
    println!("{:?}", search(Some("post".to_string()), None));
    println!("{:?}", search(None, Some("for the first".to_string())));
    println!(
        "{:?}",
        search(Some("charm".to_string()), Some("third".to_string()))
    );
}
