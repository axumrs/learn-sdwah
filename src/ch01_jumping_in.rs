use std::sync::{Arc, LazyLock};

use arc_swap::ArcSwap;
use axum::{Form, extract::Path, response::Html};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static DOGS: LazyLock<ArcSwap<Vec<Dog>>> = LazyLock::new(|| {
    ArcSwap::new(Arc::new(vec![
        Dog::new("旺财", "中华田园犬"),
        Dog::new("多多", "金毛"),
    ]))
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dog {
    pub id: Uuid,
    pub name: String,
    pub breed: String,
}

impl Dog {
    pub fn new(name: impl Into<String>, breed: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            breed: breed.into(),
        }
    }
}
pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Deserialize)]
pub struct AddDogForm {
    pub name: String,
    pub breed: String,
}

pub async fn add_dog(Form(form): Form<AddDogForm>) -> Html<String> {
    let dog = Dog::new(form.name, form.breed);

    let dogs = DOGS.load();
    let mut new_dogs = vec![dog.clone()];
    if !dogs.is_empty() {
        new_dogs.extend_from_slice(dogs.as_slice());
    }

    DOGS.store(Arc::new(new_dogs));

    let html = dog_row(&dog);
    Html(html)
}

pub async fn dog_rows() -> Html<String> {
    let dogs = (*DOGS.load()).clone();
    let html = dogs.iter().map(dog_row).collect::<Vec<_>>().join("");
    Html(html)
}

pub async fn del(Path(id): Path<Uuid>) -> Html<String> {
    let dogs = DOGS.load();
    let new_dogs = dogs.iter().filter(|dog| dog.id != id).cloned().collect();
    DOGS.store(Arc::new(new_dogs));
    Html(String::new())
}

fn dog_row(dog: &Dog) -> String {
    format!(
        r#"<tr>
    <td>{name}</td>
    <td>{breed}</td>
    <td><button class="btn-sm-icon-destructive"
    hx-delete="/api/ch01/dog/{id}"
    hx-confirm="确定删除？"
    hx-target="closest tr"
    hx-swap="delete"
    >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2 size-4"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button></td>
    </tr>"#,
        name = dog.name,
        breed = dog.breed,
        id = dog.id.to_string(),
    )
}
