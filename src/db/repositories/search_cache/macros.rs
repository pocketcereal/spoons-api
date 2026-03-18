macro_rules! define_search_cache {
    (
        entity_name: $entity_name:expr,
        get_fn: $get_fn:ident,
        cache_fn: $cache_fn:ident,
        entity_type: $entity_type:ty,
        cache_row: $cache_row_type:ty,
        table: $cache_table:ident,
        id_type: $id_type:ty,
        ids_column: $ids_column:ident,
        extract_ids: $extract_ids:expr,
        get_by_ids: $get_by_ids:expr,
        entity_id_fn: $entity_id_fn:expr,
        make_ids: $make_ids:expr,
        upsert_many: $upsert_many:expr,
        new_row: $new_row:expr,
    ) => {
        pub async fn $get_fn(
            pool: &DbPool,
            query: &str,
            limit: i32,
            offset: i32,
            cache_ttl_seconds: i64,
        ) -> Result<Option<Vec<$entity_type>>> {
            let query_hash = hash_query(query, limit, offset);
            let min_cached = min_cached_at(cache_ttl_seconds)?;
            let mut conn = get_conn(pool).await?;

            let cache_row: Option<$cache_row_type> = $cache_table::table
                .filter($cache_table::query_hash.eq(&query_hash))
                .filter($cache_table::cached_at.gt(min_cached))
                .select(<$cache_row_type>::as_select())
                .first(&mut conn)
                .await
                .optional()
                .map_err(db_error(concat!("Failed to get ", $entity_name, " search cache")))?;

            match cache_row {
                Some(row) => {
                    let ids: Vec<$id_type> = ($extract_ids)(row);
                    let entities = ($get_by_ids)(pool, &ids).await?;
                    Ok(helpers::resolve_and_order($entity_name, &ids, entities, $entity_id_fn))
                }
                None => Ok(None),
            }
        }

        pub async fn $cache_fn(
            pool: &DbPool,
            query: &str,
            limit: i32,
            offset: i32,
            entities: &[$entity_type],
        ) -> Result<()> {
            ($upsert_many)(pool, entities).await?;

            let ids: Vec<$id_type> = ($make_ids)(entities);
            let query_hash = hash_query(query, limit, offset);

            let new_cache = ($new_row)(query_hash.clone(), query.to_string(), ids, entities.len() as i64);

            let mut conn = get_conn(pool).await?;

            diesel::insert_into($cache_table::table)
                .values(&new_cache)
                .on_conflict($cache_table::query_hash)
                .do_update()
                .set((
                    $cache_table::$ids_column.eq(&new_cache.$ids_column),
                    $cache_table::total_count.eq(&new_cache.total_count),
                    $cache_table::cached_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .await
                .map_err(db_error(concat!("Failed to cache ", $entity_name, " search")))?;

            Ok(())
        }
    };
}
