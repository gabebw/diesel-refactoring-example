#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub title: Option<String>,
    pub body: Option<String>,
}
