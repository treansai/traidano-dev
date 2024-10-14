use crate::models::trade::{Side, TimeInForce, Type};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use tracing_subscriber::util::SubscriberInitExt;

pub enum AnyValue {
    U32(Option<u32>),
    Str(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Qty {
    Int(i32),
    Float(f32),
}

impl Default for Qty {
    fn default() -> Self {
        Qty::Int(0)
    }
}

// Todo: Add stop loss and take profit and order class
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub qty: Option<Qty>,
    pub national: Option<i32>,
    pub side: Side,
    #[serde(rename = "type")]
    pub order_type: Type,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<i32>,
    pub stop_price: Option<i32>,
    pub trail_price: Option<i32>,
    pub trail_percent: Option<i32>,
    pub extended_hours: Option<bool>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OrderParams {
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub after: Option<String>,
    pub util: Option<String>,
    pub direction: Option<String>,
    pub nested: Option<String>,
    pub symbols: Option<String>,
    pub side: Option<String>,
}
impl OrderParams {
    pub fn query(&self) -> String {
        let mut query = vec![];
        let query_map = self.to_hash_map();

        for key in query_map.keys() {
            match query_map.get(key) {
                Some(AnyValue::Str(Some(str_value))) => {
                    query.push(format!("{}={}", key, str_value))
                }
                Some(AnyValue::U32(Some(u_value))) => {
                    query.push(format!("{}={}", key, u_value.to_string()))
                }
                _ => {}
            }
        }

        query.sort(); // avoid some test non-determinist error
        query.join("&")
    }

    fn to_hash_map(&self) -> HashMap<&str, AnyValue> {
        let mut map = HashMap::<&str, AnyValue>::new();

        let data = self.clone();
        map.insert("status", AnyValue::Str(data.status));
        map.insert("limit", AnyValue::U32(data.limit));
        map.insert("after", AnyValue::Str(data.after));
        map.insert("util", AnyValue::Str(data.util));
        map.insert("direction", AnyValue::Str(data.direction));
        map.insert("nested", AnyValue::Str(data.nested));
        map.insert("symbols", AnyValue::Str(data.symbols));
        map.insert("side", AnyValue::Str(data.side));
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_map() {
        let params = OrderParams {
            status: Some("param1".to_string()),
            limit: Some(3_u32),
            ..OrderParams::default()
        };

        let query = params.query();

        assert_eq!(query, "limit=3&status=param1".to_string())
    }
}
