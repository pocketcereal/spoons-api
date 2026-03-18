use async_graphql::Object;

#[derive(Default)]
pub struct UnifiedQuery;

#[Object]
impl UnifiedQuery {
    async fn _unified_placeholder(&self) -> bool {
        true
    }
}
