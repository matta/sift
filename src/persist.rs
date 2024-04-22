use autosurgeon;
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
struct Task {
    pub title: String,
    pub description: String,
    #[autosurgeon(with = "option_naive_date")]
    pub due_date: Option<NaiveDate>,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
struct TodoList {
    pub tasks: Vec<Task>,
}

/// Reconcile an `Option<NaiveDate>` value with an optional string in
/// an automerge document.
///
/// This helper module is used with the #[autosurgeon(with = "option_naive_date")]
/// syntax.
mod option_naive_date {
    use autosurgeon::{Hydrate, HydrateError, Prop, ReadDoc, Reconciler};
    use chrono::NaiveDate;

    /// Create a new `Option<NaiveDate>` value from a, possibly missing,
    /// string in an automerge document.
    /// 
    /// May return an error if the string is not in the format YYYY-MM-DD
    /// or not a valid date.
    pub(super) fn hydrate<D: ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: Prop<'_>,
    ) -> Result<Option<NaiveDate>, HydrateError> {
        type OptionString = Option<String>;
        let inner = OptionString::hydrate(doc, obj, prop)?;
        match inner {
            None => Ok(None),
            Some(s) => match s.parse::<NaiveDate>() {
                Ok(d) => Ok(Some(d)),
                Err(_) => Err(HydrateError::unexpected(
                    "a valid date in YYYY-MM-DD format",
                    s,
                )),
            },
        }
    }

    // Given an `Option<NaiveDate>` value, write either a none value or
    // a string in the format YYYY-MM-DD.
    pub(super) fn reconcile<R: Reconciler>(
        date: &Option<NaiveDate>,
        mut reconciler: R,
    ) -> Result<(), R::Error> {
        match date {
            None => reconciler.none(),
            Some(d) => reconciler.str(d.format("%F").to_string()),
        }
    }
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
                    due_date: Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
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
