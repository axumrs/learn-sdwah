use arc_swap::ArcSwap;
use axum::{
    Form,
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, sync::LazyLock};
use uuid::Uuid;

static DOGS: LazyLock<ArcSwap<Vec<Dog>>> = LazyLock::new(|| {
    ArcSwap::new(Arc::new(vec![
        Dog::new("旺财", "中华田园犬"),
        Dog::new("多多", "金毛"),
    ]))
});
static SELECTED_DOG_ID: LazyLock<ArcSwap<Option<Uuid>>> =
    LazyLock::new(|| ArcSwap::new(Arc::new(None)));

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub description: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            description: description.into(),
            completed: false,
        }
    }
}

async fn get_pool() -> sqlx::Result<sqlx::SqlitePool> {
    let url = std::env::var("DATABASE_URL").unwrap_or("sqlite:todos.db".into());
    sqlx::sqlite::SqlitePoolOptions::new().connect(&url).await
}

pub struct Ch03State {
    pub pool: sqlx::SqlitePool,
}

impl Ch03State {
    pub fn new_arc(pool: sqlx::SqlitePool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

pub type Ch03ArcState = Arc<Ch03State>;

pub async fn oob_demo() -> Html<&'static str> {
    Html(
        r#"<div>新内容1</div>
    <div id="target-2" hx-swap-oob="true">新内容2</div>
    <div id="target-2" hx-swap-oob="afterend">
        <div>新内容2-之后</div>
    </div>
    <div hx-swap-oob="innerHTML:#target-3">新内容3</div>
    "#,
    )
}

pub async fn event_with_no_data() -> impl IntoResponse {
    ([("HX-Trigger", "event1")], "已分发事件1（event1）").into_response()
}

pub async fn event_with_string() -> impl IntoResponse {
    let mut trigger = HashMap::new();
    trigger.insert("event2", "AXUM.EU.ORG");
    (
        [("HX-Trigger", serde_json::to_string(&trigger).unwrap())],
        "已分发事件2（event2）",
    )
        .into_response()
}
pub async fn event_with_object() -> impl IntoResponse {
    let mut obj = HashMap::new();
    obj.insert("foo", 1);
    obj.insert("bar", 2);
    let mut trigger = HashMap::new();
    trigger.insert("event3", obj);
    (
        [("HX-Trigger", serde_json::to_string(&trigger).unwrap())],
        "已分发事件3（event3）",
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct DogInputForm {
    pub name: String,
    pub breed: String,
}

pub async fn add_dog(Form(form): Form<DogInputForm>) -> Html<String> {
    let dog = Dog::new(form.name, form.breed);

    let dogs = DOGS.load();
    let mut new_dogs = vec![dog.clone()];
    if !dogs.is_empty() {
        new_dogs.extend_from_slice(dogs.as_slice());
    }

    DOGS.store(Arc::new(new_dogs));

    let html = dog_row(&dog, false);
    Html(html)
}

pub async fn edit_dog(Path(id): Path<Uuid>, Form(form): Form<DogInputForm>) -> impl IntoResponse {
    let dog = Dog {
        id: id.clone(),
        ..Dog::new(form.name, form.breed)
    };
    let dogs = DOGS.load();
    let new_dogs = dogs
        .iter()
        .map(|d| if d.id == id { &dog } else { d })
        .cloned()
        .collect();
    DOGS.store(Arc::new(new_dogs));
    let html = dog_row(&dog, true);

    SELECTED_DOG_ID.store(Arc::new(None));

    ([("HX-Trigger", "selection-change")], Html(html)).into_response()
}

pub async fn dog_rows() -> Html<String> {
    let dogs = (*DOGS.load()).clone();
    let html = dogs
        .iter()
        .map(|d| dog_row(d, false))
        .collect::<Vec<_>>()
        .join("");
    Html(html)
}

pub async fn del_dog(Path(id): Path<Uuid>) -> Html<String> {
    let dogs = DOGS.load();
    let new_dogs = dogs.iter().filter(|dog| dog.id != id).cloned().collect();
    DOGS.store(Arc::new(new_dogs));
    Html(String::new())
}

pub async fn dog_form() -> Html<String> {
    let mut attrs = HashMap::new();
    attrs.insert("hx-on:htmx:after-request", "this.reset()".to_string());

    let selected_id = SELECTED_DOG_ID.load();
    if let Some(id) = **selected_id {
        attrs.insert("hx-put", format!("/api/ch03/dog/{id}", id = id.to_string()));
    } else {
        attrs.insert("hx-post", "/api/ch03/dog".to_string());
        attrs.insert("hx-target", "tbody".to_string());
        attrs.insert("hx-swap", "afterbegin".to_string());
    }

    let attr_str = attrs
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join(" ");

    let (name, breed) = if let Some(id) = **selected_id {
        match DOGS.load().iter().find(|dog| dog.id == id) {
            Some(v) => (v.name.clone(), v.breed.clone()),
            None => ("".to_string(), "".to_string()),
        }
    } else {
        ("".to_string(), "".to_string())
    };

    let (submit_btn_label, cancel_btn) = if (**selected_id).is_some() {
        (
            "更新",
            format!(
                r#"<button class="btn-secondary" hx-put="/api/ch03/dog/deselect" hx-swap="none" type="button">取消</button>"#
            ),
        )
    } else {
        ("添加", "".to_string())
    };
    let html = format!(
        r##"<form
        autocomplete="off"
        class="form grid gap-6"
        hx-disabled-elt="#submit-btn"
        {attr_str}
      >
        <div class="space-y-2">
          <label for="name">名字</label>
          <input type="text" id="name" name="name" value="{name}" required />
        </div>
        <div class="space-y-2">
          <label for="name">品种</label>
          <input type="text" id="breed" name="breed" value="{breed}" required />
        </div>

        <div role="group" class="button-group">
          <button type="submit" class="btn" id="submit-btn">{submit_btn_label}</button>
          {cancel_btn}
        </div>
      </form>"##
    );
    Html(html)
}

pub async fn select_dog(Path(id): Path<Uuid>) -> impl IntoResponse {
    change_selectd_id(Some(id))
}

pub async fn deselect_dog() -> impl IntoResponse {
    change_selectd_id(None)
}

fn dog_row(dog: &Dog, updating: bool) -> String {
    let mut attrs = HashMap::new();
    if updating {
        attrs.insert("hx-swap-oob", "true");
    }
    let attrs_str = attrs
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"<tr id="row-{id}" {attrs_str}>
    <td>{name}</td>
    <td>{breed}</td>
    <td>
     <div role="group" class="button-group">
      <button class="btn-sm-icon-destructive"
    hx-delete="/api/ch03/dog/{id}"
    hx-confirm="确定删除？"
    hx-target="closest tr"
    hx-swap="outerHTML"
    type="button"
    >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2 size-4"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
    <button class="btn-sm-icon" 
    hx-put="/api/ch03/dog/select/{id}"
    hx-swap="none"
    type="button"
    >
   <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil-icon lucide-pencil size-4"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg>
    </button>
    </div>
   </td>
    </tr>"#,
        name = dog.name,
        breed = dog.breed,
        id = dog.id.to_string(),
    )
}

fn change_selectd_id(id: Option<Uuid>) -> impl IntoResponse {
    SELECTED_DOG_ID.store(Arc::new(id));
    ([("HX-Trigger", "selection-change")], ()).into_response()
}

#[derive(Deserialize)]
pub struct TodoInputForm {
    pub description: String,
}
pub async fn add_todo(
    State(state): State<Ch03ArcState>,
    Form(form): Form<TodoInputForm>,
) -> impl IntoResponse {
    let m = Todo::new(form.description);
    sqlx::query("INSERT INTO ch03_todos (id,description,completed) VALUES (?,?,?)")
        .bind(&m.id)
        .bind(&m.description)
        .bind(&m.completed)
        .execute(&state.pool)
        .await
        .unwrap();

    ([("HX-Trigger", "status-change")], Html(todo_row(&m))).into_response()
}
pub async fn edit_todo(
    State(state): State<Ch03ArcState>,
    Path(id): Path<Uuid>,
    Form(form): Form<TodoInputForm>,
) -> Html<String> {
    sqlx::query("UPDATE ch03_todos SET description=? WHERE id=?")
        .bind(&form.description)
        .bind(&id)
        .execute(&state.pool)
        .await
        .unwrap();

    let m = match sqlx::query_as("SELECT id,description,completed FROM ch03_todos WHERE id=?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap()
    {
        Some(v) => v,
        None => return Html("".to_string()),
    };

    Html(todo_row(&m))
}

pub async fn todo_list(State(state): State<Ch03ArcState>) -> Html<String> {
    let todos = sqlx::query_as("SELECT id,description,completed FROM ch03_todos ORDER BY id DESC")
        .fetch_all(&state.pool)
        .await
        .unwrap();
    let html = todos.iter().map(todo_row).collect::<Vec<_>>().join("");
    Html(html)
}

pub async fn del_todo(
    State(state): State<Ch03ArcState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM ch03_todos WHERE id=?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .unwrap();
    ([("HX-Trigger", "status-change")], ()).into_response()
}
pub async fn todo_status(State(state): State<Ch03ArcState>) -> Html<String> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ch03_todos")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    let uncompleted: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ch03_todos WHERE completed=?")
        .bind(&false)
        .fetch_one(&state.pool)
        .await
        .unwrap();
    Html(format!("未完成 {uncompleted} 条 / 共 {count} 条"))
}

pub async fn todo_completed(
    State(state): State<Ch03ArcState>,
    Path((id, completed)): Path<(Uuid, bool)>,
) -> impl IntoResponse {
    sqlx::query("UPDATE ch03_todos SET completed=? WHERE id=?")
        .bind(&completed)
        .bind(&id)
        .execute(&state.pool)
        .await
        .unwrap();

    let m = match sqlx::query_as("SELECT id,description,completed FROM ch03_todos WHERE id=?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap()
    {
        Some(v) => v,
        None => return ().into_response(),
    };
    ([("HX-Trigger", "status-change")], Html(todo_row(&m))).into_response()
}

fn todo_row(todo: &Todo) -> String {
    format!(
        r#"<li class="flex items-center gap-x-4" id="item-{id}" x-data="{{completed:{completed}}}">
          <input type="checkbox"
          hx-patch="/api/ch03-todo/completed/{id}/{not_completed}"
          hx-target="closest li"
          hx-swap="outerHTML"
          :checked="completed"
           class="input shrink-0" />
          <input class="input grow" x-show="editingId === '{id}'" type="text" @click.stop="" value="{description}"
            hx-trigger="blur, keyup[keyCode === 13]"
            hx-patch="/api/ch03-todo/{id}"
            hx-swap="outerHTML"
            hx-target="closest li"
            name="description"
            @keydown.escape="editingId = ''"
            x-ref="item-{id}-input" 
           />
          <span class="grow" x-show="editingId !== '{id}'" @click.stop="!completed ? editingId = '{id}' : '';$nextTick(() => $refs['item-{id}-input'].focus());$refs['item-{id}-input'].setSelectionRange($refs['item-{id}-input'].value.length, $refs['item-{id}-input'].value.length)" 
          :class={{'text-green-600':completed,'line-through':completed}}>{description}</span>
          <button class="shrink-0 btn-sm-destructive"
          hx-confirm="确定删除？"
          hx-delete="/api/ch03-todo/{id}"
          hx-target="closest li"
          hx-swap="delete swap:50ms"
          >x</button>
        </li>"#,
        id = todo.id,
        description = todo.description,
        completed = todo.completed,
        not_completed = !todo.completed
    )
}
