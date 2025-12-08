use std::any::type_name;
use std::borrow::Borrow;
use redb::{TableDefinition, Value};

/// If an object implements Cacheable then a Cacher can be constructed for it.
///
/// - for<'a> Self: From<Self::SelfType<'a>> is required so that cache_entry.value().into() works
/// - for<'a> Self: Borrow<Self::SelfType<'a>> is required so that table.insert(key, object_to_cache.clone())?; works
pub trait Cacheable: Sized + Clone + Value + 'static
where
        for<'a> Self: From<Self::SelfType<'a>>,
        for<'a> Self: Borrow<Self::SelfType<'a>>,
{
    fn keys(&self) -> Vec<&str>;

    fn table_definition() -> TableDefinition<'static, &'static str, Self> {
        TableDefinition::new(type_name::<Self>())
    }
}