use super::TaskId;

pub(crate) trait Store {
    fn set_title(&mut self, id: TaskId, title: &str);
}
