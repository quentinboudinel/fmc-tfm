use super::{Defect, Project};

pub trait Command {
    fn execute(&mut self, project: &mut Project);
    fn undo(&mut self, project: &mut Project);
}

pub struct AddDefect {
    defect: Option<Defect>,
}

impl AddDefect {
    pub fn new(defect: Defect) -> Self {
        Self {
            defect: Some(defect),
        }
    }
}

impl Command for AddDefect {
    fn execute(&mut self, project: &mut Project) {
        if let Some(defect) = self.defect.take() {
            project.defects.push(defect);
        }
    }

    fn undo(&mut self, project: &mut Project) {
        if let Some(defect) = project.defects.pop() {
            self.defect = Some(defect);
        }
    }
}

pub struct RemoveDefect {
    index: usize,
    defect: Option<Defect>,
}

impl RemoveDefect {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            defect: None,
        }
    }
}

impl Command for RemoveDefect {
    fn execute(&mut self, project: &mut Project) {
        if self.index < project.defects.len() {
            self.defect = Some(project.defects.remove(self.index));
        }
    }

    fn undo(&mut self, project: &mut Project) {
        if let Some(defect) = self.defect.take() {
            if self.index <= project.defects.len() {
                project.defects.insert(self.index, defect);
            } else {
                project.defects.push(defect);
            }
        }
    }
}

pub struct MoveDefect {
    index: usize,
    old_pos: (f64, f64),
    new_pos: (f64, f64),
}

impl MoveDefect {
    pub fn new(index: usize, old_pos: (f64, f64), new_pos: (f64, f64)) -> Self {
        Self {
            index,
            old_pos,
            new_pos,
        }
    }
}

impl Command for MoveDefect {
    fn execute(&mut self, project: &mut Project) {
        if let Some(defect) = project.defects.get_mut(self.index) {
            defect.set_position(self.new_pos.0, self.new_pos.1);
        }
    }

    fn undo(&mut self, project: &mut Project) {
        if let Some(defect) = project.defects.get_mut(self.index) {
            defect.set_position(self.old_pos.0, self.old_pos.1);
        }
    }
}

#[derive(Default)]
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, project: &mut Project) {
        cmd.execute(project);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, project: &mut Project) -> bool {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(project);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, project: &mut Project) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(project);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PointReflector;

    fn make_point(x: f64, y: f64) -> Defect {
        Defect::PointReflector(PointReflector {
            x,
            y,
            amplitude: 1.0,
        })
    }

    #[test]
    fn add_defect_execute_and_undo() {
        let mut project = Project::default();
        let mut cmd = AddDefect::new(make_point(10.0, 20.0));

        cmd.execute(&mut project);
        assert_eq!(project.defects.len(), 1);
        assert_eq!(project.defects[0].position(), (10.0, 20.0));

        cmd.undo(&mut project);
        assert!(project.defects.is_empty());
    }

    #[test]
    fn remove_defect_execute_and_undo() {
        let mut project = Project::default();
        project.defects.push(make_point(10.0, 20.0));
        project.defects.push(make_point(30.0, 40.0));

        let mut cmd = RemoveDefect::new(0);
        cmd.execute(&mut project);
        assert_eq!(project.defects.len(), 1);
        assert_eq!(project.defects[0].position(), (30.0, 40.0));

        cmd.undo(&mut project);
        assert_eq!(project.defects.len(), 2);
        assert_eq!(project.defects[0].position(), (10.0, 20.0));
    }

    #[test]
    fn move_defect_execute_and_undo() {
        let mut project = Project::default();
        project.defects.push(make_point(10.0, 20.0));

        let mut cmd = MoveDefect::new(0, (10.0, 20.0), (50.0, 60.0));
        cmd.execute(&mut project);
        assert_eq!(project.defects[0].position(), (50.0, 60.0));

        cmd.undo(&mut project);
        assert_eq!(project.defects[0].position(), (10.0, 20.0));
    }

    #[test]
    fn command_history_undo_redo() {
        let mut project = Project::default();
        let mut history = CommandHistory::new();

        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.execute(Box::new(AddDefect::new(make_point(10.0, 20.0))), &mut project);
        assert_eq!(project.defects.len(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut project);
        assert!(project.defects.is_empty());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo(&mut project);
        assert_eq!(project.defects.len(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn new_command_clears_redo_stack() {
        let mut project = Project::default();
        let mut history = CommandHistory::new();

        history.execute(Box::new(AddDefect::new(make_point(10.0, 20.0))), &mut project);
        history.undo(&mut project);
        assert!(history.can_redo());

        history.execute(Box::new(AddDefect::new(make_point(30.0, 40.0))), &mut project);
        assert!(!history.can_redo());
    }

    #[test]
    fn multiple_undo_redo_sequence() {
        let mut project = Project::default();
        let mut history = CommandHistory::new();

        history.execute(Box::new(AddDefect::new(make_point(1.0, 1.0))), &mut project);
        history.execute(Box::new(AddDefect::new(make_point(2.0, 2.0))), &mut project);
        history.execute(Box::new(AddDefect::new(make_point(3.0, 3.0))), &mut project);
        assert_eq!(project.defects.len(), 3);

        history.undo(&mut project);
        history.undo(&mut project);
        assert_eq!(project.defects.len(), 1);
        assert_eq!(project.defects[0].position(), (1.0, 1.0));

        history.redo(&mut project);
        assert_eq!(project.defects.len(), 2);
        assert_eq!(project.defects[1].position(), (2.0, 2.0));
    }
}
