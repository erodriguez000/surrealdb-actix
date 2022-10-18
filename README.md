# Actix REST API using SurrealDB

A simple example for integrating SurrealDB into your Actix web server.

Big thank you to [Jeremy Chone](https://github.com/jeremychone) and [Demola Malomo](https://blog.devgenius.io/build-a-rest-api-with-rust-and-mongodb-actix-web-version-a275215c262a)

## Dependencies

```
[dependencies]
actix-web = "4"
serde = { version = "1.0.136", features = ["derive"]}
futures = "0.3"
surrealdb = "1.0.0-beta.8"
thiserror = "1"
```

## Tips

Install [cargo watch](https://crates.io/crates/cargo-watch) and run your binary with the following command for hot reloading on saves.

```
cargo watch -q -c -x run
```

Note: The rust-analyzer VS Code extension was causing huge compile time delays (at least on my laptop), I had to disable it and rely on console error logs to effectively develop with the surrealdb crate.

## main.rs

Initialize the SurrealDB instance (which is created in ./repository/surrealdb_repo.rs) and wrap it in an actix-web::web::Data type.

```rs

    let surreal = SurrealDBRepo::init().await.expect("Error connecting to SurrealDB!");
    
    let db_data = Data::new(surreal);
    
```

Clone that instance in the app_data() method to send to the api routes.

```rs

    HttpServer::new(move || { 
        App::new()
            .app_data(db_data.clone())
            .service(create_todo)
            .service(get_todos)
            .service(get_todo)
            .service(update_todo)
            .service(delete_todo)
        })

```

## surrealdb_repo.rs

Imports

```rs
use std::sync::Arc;
use surrealdb::{Datastore, Session, Error};
use surrealdb::sql::{Object, Value, Array, thing};
```

Public traits to allow incoming data to be converted to our type, then into a surrealdb::Value

```rs
pub trait Creatable: Into<Value> {}
pub trait Patchable: Into<Value> {}
```

The clone trait must be implemented and Datastore sent wrapped in an atomic reference counter to allow the datastore to be sent across routes

```rs
#[derive(Clone)]
pub struct SurrealDBRepo {
    pub ds: Arc<Datastore>,
    pub ses: Session
}
```

Here we create a local file to store our data, the current options for connecting to a Surreal DB instance are as follows:

```rs
// As a file in the local directory

let ds = Arc::new(Datastore::new("file://surreal.db").await?);

// In memory

let ds = Arc::new(Datastore::new("memory").await?);

// To a TiKv server

let ds = Arc::new(Datastore::new("tikv://127.0.0.1:2379").await?);

// Other: Making Http requests to your server's endpoint with name space, database, and credentials (if applicable) ex: http://localhost:8000/sql is where all Http requests will go.

```

Set the session with the name space and database you want to use.

```rs
impl SurrealDBRepo {
    pub async fn init() -> Result<Self, Error> {

        let ds = Arc::new(Datastore::new("file://surreal.db").await?);
        let ses = Session::for_kv().with_ns("test").with_db("test");

        //let ds = Arc::new(ds);
        Ok(SurrealDBRepo { ses, ds })
    }
}
```

### todo_model.rs

Two structs exist for Todos, a main one for creating, and a TodoPatch for updates.

```rs

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Option<String>,
    pub title: String,
    pub body: String,
}

impl Creatable for Todo{}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoPatch {
    pub title: Option<String>,
    pub body: Option<String>,
}

impl Patchable for TodoPatch {}

```

Each struct needs to implment From<Self> for [surrealdb::Value](https://docs.rs/surrealdb/1.0.0-beta.8/surrealdb/sql/enum.Value.html) ex for TodoPatch

```rs
impl From<TodoPatch> for Value {
    fn from(val: TodoPatch) -> Self {

        let mut value: BTreeMap<String, Value> = BTreeMap::new();
        
        if let Some(t) = val.title {
            value.insert("title".into(), t.into());
        }

        if let Some(b) = val.body {
            value.insert("body".into(), b.into());
        }
        Value::from(value)
    }
}

```

Note: surrealdb:Value is how surreal DB will consume your data, you'll need to convert query variables into Value. The response from a query will be Result<Value, Error>. Again, this value will need to be converted to a consumable rust type. In this example, the ```surrealdb::Value```s returned are converted to ```surrealdb::Object```s, which are JSON serializable and can be sent directly in the Http response. 

This is done by implementing rust's new type pattern:

```rs

// from: https://github.com/rust-awesome-app/template-app-base/blob/main/src-tauri/src/store/try_froms.rs

pub struct W<T>(pub T);

impl TryFrom<W<Value>> for Object {
	type Error = Error;
	fn try_from(val: W<Value>) -> Result<Object> {
		match val.0 {
			Value::Object(obj) => Ok(obj),
			_ => Err(Error::XValueNotOfType("Object")),
		}
	}
}

impl TryFrom<W<Value>> for Array {
	type Error = Error;
	fn try_from(val: W<Value>) -> Result<Array> {
		match val.0 {
			Value::Array(obj) => Ok(obj),
			_ => Err(Error::XValueNotOfType("Array")),
		}
	}
}

impl TryFrom<W<Value>> for i64 {
	type Error = Error;
	fn try_from(val: W<Value>) -> Result<i64> {
		match val.0 {
			Value::Number(obj) => Ok(obj.as_int()),
			_ => Err(Error::XValueNotOfType("i64")),
		}
	}
}

impl TryFrom<W<Value>> for bool {
	type Error = Error;
	fn try_from(val: W<Value>) -> Result<bool> {
		match val.0 {
			Value::False => Ok(false),
			Value::True => Ok(true),
			_ => Err(Error::XValueNotOfType("bool")),
		}
	}
}

impl TryFrom<W<Value>> for String {
	type Error = Error;
	fn try_from(val: W<Value>) -> Result<String> {
		match val.0 {
			Value::Strand(strand) => Ok(strand.as_string()),
			Value::Thing(thing) => Ok(thing.to_string()),
			_ => Err(Error::XValueNotOfType("String")),
		}
	}
}

```

From here, we can simply manage our Todos with our TodoBMC struct (backend model controller) in todo_model.rs. We expect an actix-web::Data<SurrealDBRepo> struct to be passed to us via the route, which will manage the queries. We set up when we instantiated SurrealDBRepo in main.rs and cloned it in the app_data() method).

```rs
pub struct TodoBMC;

impl TodoBMC {
    /* snip */
    pub async fn get(db: Data<SurrealDBRepo>, tid: &str) -> Result<Object, Error> {
        let sql = "SELECT * FROM $th";
            
            let tid = format!("todo:{}", tid);

            let vars: BTreeMap<String, Value> = map!["th".into() => thing(&tid)?.into()];
    
            let ress = db.ds.execute(sql, &db.ses, Some(vars), true).await?;
    
            let first_res = ress.into_iter().next().expect("Did not get a response");
    
            W(first_res.result?.first()).try_into()
           
    }
    /* snip */
}

```

### todo_api.rs

Get the id string from the route, then, using the TodoBMC struct, query the data for the id. Respond with the todo if there is no error, respond with an Http error on failure.

```rs

#[get("/todos/{id}")]
pub async fn get_todo(db: Data<SurrealDBRepo>, path: Path<String>) -> HttpResponse {
    let id = path.into_inner();
    
    if id.is_empty() {
        return HttpResponse::BadRequest().body("invalid ID");
    }
    
    let todo_detail = TodoBMC::get(db, &id).await;
    
    match todo_detail {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

```

Hope this was helpful!

## Project Structure

```
.
├── Cargo.lock
├── Cargo.toml
├── src
│   ├── api
│   │   ├── mod.rs
│   │   └── todo_api.rs
│   ├── error.rs
│   ├── main.rs
│   ├── model
│   │   ├── mod.rs
│   │   └── todo_model.rs
│   ├── prelude.rs
│   ├── repository
│   │   ├── mod.rs
│   │   └── surrealdb_repo.rs
│   └── utils
│       ├── macros.rs
│       ├── mod.rs
│       └── try_froms.rs

├── surreal.db
│   ├── ...
|
└── target
    ├── ...
```