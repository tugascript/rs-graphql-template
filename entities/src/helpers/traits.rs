use sea_orm::{EntityTrait, ModelTrait, Select};

use crate::enums::{CursorEnum, OrderEnum};

pub trait GQLQuery: EntityTrait {
    fn query(
        order: OrderEnum,
        cursor: CursorEnum,
        after: Option<String>,
        search: Option<String>,
    ) -> (Select<Self>, Option<Select<Self>>);
}

pub trait GQLAfter: ModelTrait {
    fn after(&self, cursor: CursorEnum) -> String;
}
