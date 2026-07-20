/// Test that query_items filters by type_id correctly.
#[tokio::test]
async fn test_query_filter_type_id() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    // Insert test type + items
    sqlx::query("INSERT INTO item_types (id, name, icon) VALUES (10, 'Test', '📦')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('a001', 'Alpha', 1, '{}', 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('a002', 'Beta', 10, '{}', 'default', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z')")
        .execute(&pool).await.unwrap();

    let filter = FilterNode::Condition {
        field: "type_id".to_string(),
        op: "=".to_string(),
        value: serde_json::json!(1),
    };

    let result = index_lib::commands::query::execute_query(&pool, &filter, None, None, None)
        .await
        .expect("query failed");

    assert_eq!(result.total, 1);
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].id, "a001");
}

/// Test that query_items filters by name contains.
#[tokio::test]
async fn test_query_name_contains() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('b001', 'Hello World', 1, '{}', 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('b002', 'Goodbye', 1, '{}', 'default', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z')")
        .execute(&pool).await.unwrap();

    let filter = FilterNode::Condition {
        field: "name".to_string(),
        op: "contains".to_string(),
        value: serde_json::json!("Hello"),
    };

    let result = index_lib::commands::query::execute_query(&pool, &filter, None, None, None)
        .await
        .expect("query failed");

    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].name, "Hello World");
}

/// Test property filtering with json_extract.
#[tokio::test]
async fn test_query_property_equals() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;
    use serde_json::json;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('c001', 'Item1', 1, ?, 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"status": "done", "rating": 5})).unwrap())
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('c002', 'Item2', 1, ?, 'default', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"status": "todo", "rating": 3})).unwrap())
        .execute(&pool).await.unwrap();

    let filter = FilterNode::Condition {
        field: "status".to_string(),
        op: "=".to_string(),
        value: json!("done"),
    };

    let result = index_lib::commands::query::execute_query(&pool, &filter, None, None, None)
        .await
        .expect("query failed");

    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].id, "c001");
}

/// Test nested AND/OR logic.
#[tokio::test]
async fn test_query_nested_and_or() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;
    use serde_json::json;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('d001', 'A', 1, ?, 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"class": "娱乐", "rating": 5})).unwrap())
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('d002', 'B', 1, ?, 'default', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"class": "学习", "rating": 2})).unwrap())
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('d003', 'C', 1, ?, 'default', '2024-01-03T00:00:00Z', '2024-01-03T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"class": "运动", "rating": 4})).unwrap())
        .execute(&pool).await.unwrap();

    // (class = "娱乐" OR class = "学习") AND rating > 3
    let filter = FilterNode::Logic {
        and: Some(vec![
            FilterNode::Logic {
                or: Some(vec![
                    FilterNode::Condition { field: "class".to_string(), op: "=".to_string(), value: json!("娱乐") },
                    FilterNode::Condition { field: "class".to_string(), op: "=".to_string(), value: json!("学习") },
                ]),
                and: None,
            },
            FilterNode::Condition { field: "rating".to_string(), op: ">".to_string(), value: json!(3) },
        ]),
        or: None,
    };

    let result = index_lib::commands::query::execute_query(&pool, &filter, None, None, None)
        .await
        .expect("query failed");

    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].id, "d001");
}

/// Test extract returns requested property values.
#[tokio::test]
async fn test_query_extract() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;
    use serde_json::json;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('e001', 'X', 1, ?, 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"score": 90, "level": "A"})).unwrap())
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('e002', 'Y', 1, ?, 'default', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"score": 60, "level": "B"})).unwrap())
        .execute(&pool).await.unwrap();

    let filter = FilterNode::Condition {
        field: "type_id".to_string(),
        op: "=".to_string(),
        value: json!(1),
    };

    let result = index_lib::commands::query::execute_query(
        &pool, &filter, Some(&vec!["score".to_string()]), None, None,
    ).await.expect("query failed");

    assert_eq!(result.total, 2);
    let extracted = result.extracted.expect("extracted should be Some");
    assert_eq!(extracted.get("e001").unwrap().get("score").unwrap(), &json!(90));
    assert_eq!(extracted.get("e002").unwrap().get("score").unwrap(), &json!(60));
    assert!(extracted.get("e001").unwrap().get("level").is_none());
}

/// Test is_null and is_not_null operators.
#[tokio::test]
async fn test_query_is_null() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;
    use serde_json::json;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('f001', 'Has', 1, ?, 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .bind(serde_json::to_string(&json!({"note": "hello"})).unwrap())
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at) VALUES ('f002', 'Missing', 1, '{}', 'default', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z')")
        .execute(&pool).await.unwrap();

    let filter = FilterNode::Condition {
        field: "note".to_string(),
        op: "is_null".to_string(),
        value: serde_json::Value::Null,
    };

    let result = index_lib::commands::query::execute_query(&pool, &filter, None, None, None)
        .await.expect("query failed");

    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].id, "f002");
}

/// Test invalid field name is rejected.
#[tokio::test]
async fn test_query_invalid_field() {
    use sqlx::sqlite::SqlitePool;
    use index_lib::models::FilterNode;

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let filter = FilterNode::Condition {
        field: "1; DROP TABLE items--".to_string(),
        op: "=".to_string(),
        value: serde_json::json!("x"),
    };

    let result = index_lib::commands::query::execute_query(&pool, &filter, None, None, None).await;
    assert!(result.is_err());
}
