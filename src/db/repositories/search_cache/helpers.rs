use std::collections::HashMap;

pub fn all_ids_resolved(entity: &str, expected: usize, actual: usize) -> bool {
    if actual < expected {
        tracing::warn!(
            entity = entity,
            expected = expected,
            actual = actual,
            "Some cached {} IDs did not resolve — treating as cache miss",
            entity,
        );
        false
    } else {
        true
    }
}

pub fn resolve_and_order<T: Clone, Id: std::fmt::Display>(
    entity_name: &str,
    ids: &[Id],
    entities: Vec<T>,
    id_fn: impl Fn(&T) -> String,
) -> Option<Vec<T>> {
    let by_id: HashMap<String, T> = entities.into_iter().map(|e| (id_fn(&e), e)).collect();
    let ordered: Vec<T> = ids
        .iter()
        .filter_map(|id| by_id.get(&id.to_string()).cloned())
        .collect();
    if !all_ids_resolved(entity_name, ids.len(), ordered.len()) {
        return None;
    }
    Some(ordered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_and_order_preserves_id_order() {
        let entities = vec![("b", 2), ("a", 1), ("c", 3)];
        let ids = vec!["a", "b", "c"];
        let result = resolve_and_order("test", &ids, entities, |e| e.0.to_string());
        assert_eq!(result, Some(vec![("a", 1), ("b", 2), ("c", 3)]));
    }

    #[test]
    fn resolve_and_order_returns_none_on_missing_ids() {
        let entities = vec![("a", 1)];
        let ids = vec!["a", "b"];
        let result = resolve_and_order("test", &ids, entities, |e| e.0.to_string());
        assert_eq!(result, None);
    }

    #[test]
    fn resolve_and_order_handles_empty() {
        let entities: Vec<(&str, i32)> = vec![];
        let ids: Vec<&str> = vec![];
        let result = resolve_and_order("test", &ids, entities, |e| e.0.to_string());
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn resolve_and_order_works_with_numeric_ids() {
        let entities = vec![(2, "b"), (1, "a"), (3, "c")];
        let ids: Vec<i64> = vec![1, 2, 3];
        let result = resolve_and_order("test", &ids, entities, |e| e.0.to_string());
        assert_eq!(result, Some(vec![(1, "a"), (2, "b"), (3, "c")]));
    }
}
