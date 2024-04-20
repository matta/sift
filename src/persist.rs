use autosurgeon;

use crate::app::SerializableNaiveDate;

#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
struct Task {
    pub title: String,
    pub description: String,
    pub due_date: Option<SerializableNaiveDate>,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
struct TodoList {
    pub tasks: Vec<Task>,
}

#[cfg(test)]
mod tests {
    use automerge::ScalarValue;
    use automerge_test::{assert_doc, list, map};

    use super::*;

    #[test]
    fn test() {
        let todo_list = TodoList {
            tasks: vec![
                Task {
                    title: "first title".to_string(),
                    description: "first description".to_string(),
                    due_date: Some(SerializableNaiveDate::from_naive_date(
                        chrono::naive::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                    )),
                    completed: false,
                },
                Task {
                    title: "first title".to_string(),
                    description: "first description".to_string(),
                    due_date: None,
                    completed: false,
                },
            ],
        };

        let mut doc = automerge::AutoCommit::new();
        autosurgeon::reconcile(&mut doc, &todo_list).unwrap();

        assert_doc!(
            doc.document(),
            map! {
                "tasks" => {list![
                    {map! {
                        "title" => {"first title"},
                        "description" => {"first description"},
                        "due_date" => {"2022-01-01"},
                        "completed" => {false},
                    }},
                    {map! {
                        "title" => {"first title"},
                        "description" => {"first description"},
                        "due_date" => {ScalarValue::Null},
                        "completed" => {false},
                    }},
                ]},
            }
        );

        let todo_list2: TodoList = autosurgeon::hydrate(&doc).unwrap();
        assert_eq!(todo_list, todo_list2);
    }
}
