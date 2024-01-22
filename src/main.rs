use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use dioxus::fullstack::prelude::*;
use dioxus::prelude::*;
use uuid::Uuid;

fn main() {
    LaunchBuilder::new()
        .with_cfg(desktop!(dioxus::desktop::Config::new().with_window(
            dioxus::desktop::WindowBuilder::new().with_always_on_top(true)
        )))
        .launch(app);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    created: u64,
    contents: String,
    completed: bool,
}

type Todos = HashMap<Uuid, Todo>;

fn app() -> Element {
    let mut todos = use_signal::<Todos>(|| {
        std::fs::read_to_string("todos.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| HashMap::new())
    });

    use_effect(move || {
        // Do a local sync
        std::fs::write("todos.json", serde_json::to_string(&*todos.read()).unwrap()).unwrap();

        // And sync to remote db
        spawn(async move {
            // sync_todos(todos()).await.unwrap();
            println!("Synced todos to remote db");
        });
    });

    let sorted_todos = use_memo(move || {
        let mut sorted = todos.read().keys().cloned().collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            let a = todos.read().get(a).unwrap().created;
            let b = todos.read().get(b).unwrap().created;
            a.cmp(&b)
        });
        sorted
    });

    let mut current_todo_input = use_signal(|| "".to_string());

    let mut insert_todo = move || {
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let todo = Todo {
            id: Uuid::new_v4(),
            contents: current_todo_input(),
            completed: false,
            created,
        };
        todos.write().insert(todo.id, todo);
        current_todo_input.set("".to_string());
    };

    rsx! {
        h1 { "Welcome to my cool todo app" }
        div {
            div {
                input {
                    onmounted: move |e| {
                        e.inner().set_focus(true);
                    },
                    r#type: "text",
                    value: current_todo_input(),
                    oninput: move |e| current_todo_input.set(e.value()),
                    onkeypress: move |e| {
                        if e.key() == Key::Enter {
                            insert_todo();
                        }
                    }
                }
            }

            div {
                for id in sorted_todos.read().iter().cloned() {
                    div { class: "todo",
                        button { onclick: move |_| { todos.write().remove(&id); }, "X" }
                        input {
                            r#type: "checkbox",
                            checked: todos.read()[&id].completed,
                            oninput: move |e| todos.write().get_mut(&id).unwrap().completed = e.checked(),
                        }
                        span { "{todos.read()[&id].contents}" }
                    }
                }
            }
        }
    }
}

// #[server_fn::server(Api, "/sync")]
// pub async fn sync_todos(todos: Todos) -> Result<(), ServerFnError> {
//     println!("Syncing todos to sqlite... {todos:?}");
//     Ok(())
// }
