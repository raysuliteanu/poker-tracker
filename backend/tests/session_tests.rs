mod common;
use common::TestDb;
use diesel::{prelude::*, sql_types::Integer};
use rstest::rstest;

use crate::common::test_db;

#[rstest]
#[tokio::test]
async fn rstest_test_database_connection_and_isolation(#[future] test_db: TestDb) {
    let db = test_db.await;
    let mut conn = db.get_connection();
    let result = diesel::select(diesel::dsl::sql::<Integer>("1")).execute(&mut conn);
    assert!(result.is_ok());
}
