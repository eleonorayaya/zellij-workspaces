use zellij_tile::prelude::*;

use crate::workspaces::Workspace;

#[derive(Default)]
pub struct Picker {
    cursor: usize,
    workspaces: Vec<Workspace>,
}

impl From<Vec<Workspace>> for Picker {
    fn from(workspaces: Vec<Workspace>) -> Self {
        Self {
            cursor: 0,
            workspaces: workspaces,
        }
    }
}

impl Picker {
    // TODO: sort workspaces by name or last used
    pub fn render(&self, rows: usize, cols: usize) {
        let workspace_count = self.workspaces.len();

        let from = self
            .cursor
            .saturating_sub(rows.saturating_sub(1) / 2)
            .min(workspace_count.saturating_sub(rows));

        let missing_rows = rows.saturating_sub(workspace_count);

        if missing_rows > 0 {
            for _ in 0..missing_rows {
                println!();
            }
        }

        self.workspaces
            .clone()
            .into_iter()
            .enumerate()
            .for_each(|(i, workspace)| {
                let label = workspace.name;
                let len = label.len();
                let text_len = label.len();

                let item = Text::new(label);
                let item = match i == self.cursor {
                    true => item.color_range(0, 0..text_len).selected(),
                    false => item,
                };

                print_text(item);
                println!();
            });
    }

    pub fn get_selection(&self) -> Option<Workspace> {
        if self.cursor < self.workspaces.len() {
            Some(self.workspaces[self.cursor].clone())
        } else {
            None
        }
    }

    pub fn handle_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = self.workspaces.len() - 1;
        }
    }

    pub fn handle_down(&mut self) {
        if self.cursor < self.workspaces.len() - 1 {
            self.cursor += 1;
        } else {
            self.cursor = 0;
        }
    }
}

