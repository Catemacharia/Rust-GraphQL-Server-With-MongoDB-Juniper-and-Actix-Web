
use juniper::{RootNode,FieldResult};
use dotenv::dotenv;
use mongodb::{
    bson::doc,
    sync::Client,
};
use serde::{Serialize,Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    title: String,
    description: String,
    completed: bool
}

fn connect_to_db()-> FieldResult<Client> {
    dotenv().ok();
    let db_url = std::env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    let client = Client::with_uri_str(&db_url)?;
    return Ok(client);
}

#[juniper::object(description = "A todo")]
impl Todo{
    pub fn title(&self)->&str{
        self.title.as_str()
    }

    pub fn description(&self)->&str{
        self.description.as_str()
    }

    pub fn completed(&self)->bool{
        self.completed
    }
}


pub struct QueryRoot;

#[juniper::object]
impl QueryRoot {
    fn todos() -> FieldResult<Vec<Todo>> {
        let client = connect_to_db()?;
        let collection = client.database("todos").collection("todos");
        let cursor = collection.find(None, None).unwrap();
        let mut todos = Vec::new();
        for result in cursor {
            todos.push(result?);
        }
        return Ok({
            todos
        })
    }
}

pub struct MutationRoot;

#[derive(juniper::GraphQLInputObject,Debug, Clone)]
pub struct NewTodo{
    pub title: String,
    pub description: String,
    pub completed: bool
}
#[juniper::object]
impl MutationRoot {
    fn create_todo(new_todo: NewTodo) -> FieldResult<Todo> {
        let client = connect_to_db()?;
        let collection = client.database("todos").collection("todos");
        let todo = doc!{
            "title": new_todo.title,
            "description": new_todo.description,
            "completed": new_todo.completed
        };
        let result = collection.insert_one(todo, None).unwrap();
        let id = result.inserted_id.as_object_id().unwrap().to_hex();
        let inserted_todo = collection.find_one(Some(doc!{"_id": id}), None).unwrap().unwrap();
        return Ok(Todo{
            title: inserted_todo.get("title").unwrap().as_str().unwrap().to_string(),
            description: inserted_todo.get("description").unwrap().as_str().unwrap().to_string(),
            completed: inserted_todo.get("completed").unwrap().as_bool().unwrap()
        });     
    }
}


pub type Schema = RootNode<'static, QueryRoot, MutationRoot,>;

pub fn create_schema() -> Schema {
    return Schema::new(QueryRoot, MutationRoot);
}