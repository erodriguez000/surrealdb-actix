use std::sync::Arc;
use surrealdb::sql::Value;
use surrealdb::{Datastore, Session, Error};

pub trait Creatable: Into<Value> {}
pub trait Patchable: Into<Value> {}

#[derive(Clone)]
pub struct SurrealDBRepo {
    pub ds: Arc<Datastore>,
    pub ses: Session
}

impl SurrealDBRepo {
    pub async fn init() -> Result<Self, Error> {
        let ds = Arc::new(Datastore::new("file://surreal.db").await?);
        
        let ses = Session::for_kv().with_ns("test").with_db("test");

        Ok(SurrealDBRepo { ses, ds })
    }
}