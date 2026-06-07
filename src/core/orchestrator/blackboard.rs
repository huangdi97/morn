//! blackboard — Coordinates team work through shared blackboard-style context.
use super::{Orchestrator, TeamMemberOutput};

impl Orchestrator {
    pub(super) fn run_blackboard(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut board = format!("[Blackboard] Initial: {}\n", input);
        let mut outputs = Vec::new();
        for member in members {
            let result = self.dispatch_agent(member, self.read_blackboard(&board))?;
            self.post_to_blackboard(&mut board, member, &result.output);
            outputs.push(result);
        }
        self.clear_blackboard(&mut board);
        Ok(outputs)
    }

    pub(super) fn post_to_blackboard(&self, board: &mut String, member: &str, output: &str) {
        board.push_str(&format!("{}: {}\n", member, output));
    }

    pub(super) fn read_blackboard<'a>(&self, board: &'a str) -> &'a str {
        board
    }

    pub(super) fn clear_blackboard(&self, board: &mut String) {
        board.clear();
    }
}
