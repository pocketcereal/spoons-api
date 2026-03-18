use rand::seq::SliceRandom;

pub(crate) fn random_sample<T>(mut items: Vec<T>, n: usize) -> Vec<T> {
    if items.len() <= n {
        return items;
    }
    items.partial_shuffle(&mut rand::thread_rng(), n);
    items.truncate(n);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_sample_fewer_than_n() {
        let items = vec![1, 2, 3];
        let result = random_sample(items, 10);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_random_sample_exact_n() {
        let items = vec![1, 2, 3];
        let result = random_sample(items, 3);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_random_sample_larger_than_n() {
        let items: Vec<i32> = (0..100).collect();
        let result = random_sample(items, 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_random_sample_empty() {
        let items: Vec<i32> = vec![];
        let result = random_sample(items, 5);
        assert!(result.is_empty());
    }
}
