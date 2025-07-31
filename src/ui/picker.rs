use zellij_tile::prelude::*;

use crate::workspaces::Workspace;

pub struct Picker<'a> {
    cursor: usize,
    workspaces: Vec<Workspace<'a>>,
}

impl<'a: 'b, 'b> From<Vec<Workspace<'a>>> for Picker<'b> {
    fn from(workspaces: Vec<Workspace<'a>>) -> Self {
        Self {
            cursor: 0,
            workspaces: workspaces,
        }
    }
}

impl Picker<'_> {
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
}

