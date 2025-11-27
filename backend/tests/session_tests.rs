mod common;
use common::TestDb;
use diesel::{prelude::*, sql_types::Integer};
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_database_connection_and_isolation() {
    let db = TestDb::new().await;
    let mut conn = db.get_connection();
    let result = diesel::select(diesel::dsl::sql::<Integer>("1")).execute(&mut conn);
    assert!(result.is_ok());
}
