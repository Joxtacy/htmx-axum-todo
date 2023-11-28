use askama::Template;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "list-item.html")]
pub struct ListItem<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "error-modal.html")]
pub struct ErrorModal<'a> {
    pub error_message: &'a str,
}

#[derive(Template)]
#[template(path = "success-modal.html")]
pub struct SuccessModal<'a> {
    pub message: &'a str,
}

#[derive(Template)]
#[template(path = "validate-todo.html")]
pub struct ValidateTodoModal<'a> {
    pub error_message: &'a str,
    pub value: &'a str,
}

#[derive(Template)]
#[template(path = "todo.html")]
pub struct TodoItem<'a> {
    pub title: &'a str,
    pub id: Uuid,
    pub done: bool,
}

#[derive(Template)]
#[template(path = "edit-todo.html")]
pub struct EditTodoItem<'a> {
    pub title: &'a str,
    pub id: Uuid,
    pub done: bool,
}
