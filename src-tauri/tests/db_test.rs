/// Test that migrations run successfully against an in-memory SQLite database
/// and that the preset data inserted by the migration is correct.
#[tokio::test]
async fn test_migration_preset_data() {
    use sqlx::sqlite::SqlitePool;

    // Connect to an in-memory SQLite database
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory SQLite");

    // Run migrations – the macro embeds the SQL files at compile time.
    // The path is relative to CARGO_MANIFEST_DIR, i.e. src-tauri/
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    // Assert there are exactly 2 item types
    let type_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM item_types")
        .fetch_one(&pool)
        .await
        .expect("query failed");
    assert_eq!(type_count.0, 2, "expected 2 preset item types");

    // Assert there are exactly 4 fields
    let field_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fields")
        .fetch_one(&pool)
        .await
        .expect("query failed");
    assert_eq!(field_count.0, 4, "expected 4 preset fields");

    // Verify the names and icons of the item types
    let types: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT id, name, icon FROM item_types ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("query failed");
    assert_eq!(types.len(), 2);
    assert_eq!(types[0].1, "通用");
    assert_eq!(types[0].2, "file");
    assert_eq!(types[1].1, "任务");
    assert_eq!(types[1].2, "checkbox");

    // Verify the fields
    let fields: Vec<(i64, i64, String, String, i32)> =
        sqlx::query_as("SELECT id, type_id, name, field_type, position FROM fields ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("query failed");
    assert_eq!(fields.len(), 4);
    // 通用.备注 text
    assert_eq!(fields[0].1, 1);
    assert_eq!(fields[0].2, "备注");
    assert_eq!(fields[0].3, "text");
    // 任务.描述 text
    assert_eq!(fields[1].1, 2);
    assert_eq!(fields[1].2, "描述");
    assert_eq!(fields[1].3, "text");
    // 任务.完成 checkbox
    assert_eq!(fields[2].1, 2);
    assert_eq!(fields[2].2, "完成");
    assert_eq!(fields[2].3, "checkbox");
    // 任务.截止日 date
    assert_eq!(fields[3].1, 2);
    assert_eq!(fields[3].2, "截止日");
    assert_eq!(fields[3].3, "date");
}
