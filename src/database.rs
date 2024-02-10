use std::path::Path;

use rusqlite::{types::Value, Row};
use serde::Serialize;
pub use rusqlite::Connection;

pub fn create_connection<P: AsRef<Path>>(db: P) -> Connection {
    Connection::open(db).expect("Failed to establish a connection")
}

#[derive(Serialize)]
pub struct Cell {
    pub is_null: bool,
    pub is_blob: bool,
    pub value: String,
}

pub struct QueryAnswer<'a> {
    pub sql: &'a str,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
}

impl<'a> QueryAnswer<'a> {
    pub fn from_sql(sql: &'a str, conn: &Connection) -> Self {
        let mut prepared = conn.prepare(sql).expect("Invalid query");

        let num_cols = prepared.column_count();

        let rows = prepared
            .query_map([], |row| Ok(Self::create_vec_row(row, num_cols)))
            .expect("An error occured")
            .collect::<Result<Vec<_>, _>>()
            .expect("An error occured");

        let headers = prepared
            .column_names()
            .into_iter()
            .map(str::to_string)
            .collect();

        Self { sql, headers, rows }
    }

    fn create_vec_row(row: &Row, num_cols: usize) -> Vec<Cell> {
        (0..num_cols)
            .map(|idx| {
                let value: Value = row.get_ref_unwrap(idx).into();

                let is_null = matches!(value, Value::Null);
                let is_blob = matches!(value, Value::Blob(_));

                let rendered = match value {
                    Value::Null => "(NULL)".to_string(),
                    Value::Blob(_) => "(BLOB)".to_string(),
                    Value::Text(val) => val,
                    Value::Real(val) => val.to_string(),
                    Value::Integer(val) => val.to_string(),
                };

                Cell {
                    is_null,
                    is_blob,
                    value: rendered,
                }
            })
            .collect()
    }
}
