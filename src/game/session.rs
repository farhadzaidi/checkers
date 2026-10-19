use super::board::Board;
use super::hash::PositionHash;
use super::movegen::generate_moves;
use super::moves::{Move, MoveList};
use super::outcome::{GameOutcome, game_outcome};

/// One game being played, with every position kept for undo and redo
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameSession {
    current: usize, // Index of the position on the board right now.
    positions: Vec<Board>,
    played: Vec<Move>, // played[i] leads from positions[i] to positions[i + 1]
}

impl GameSession {
    /// Start a game from the starting position
    pub fn new() -> Self {
        Self::from_board(Board::start_position())
    }

    /// Start a game from any position
    pub fn from_board(board: Board) -> Self {
        Self {
            current: 0,
            positions: vec![board],
            played: Vec::new(),
        }
    }

    /// Start a game from a position in PDN FEN
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let board = Board::from_fen(fen)?;
        Ok(Self::from_board(board))
    }

    /// Return the position on the board right now
    pub fn current_position(&self) -> Board {
        self.positions[self.current]
    }

    /// Return the legal moves in the current position
    pub fn moves(&self) -> MoveList {
        generate_moves(&self.current_position())
    }

    /// Play a legal move in the current position
    pub fn play(&mut self, mv: Move) {
        debug_assert!(
            self.outcome().is_none(),
            "{mv} was played after the game ended"
        );
        debug_assert!(
            self.moves().contains(&mv),
            "{mv} is not legal in {}",
            self.current_position().to_fen()
        );

        let mut next = self.current_position();
        next.make_move(mv);

        // Playing after an undo starts a new branch, so the old redo positions go
        self.positions.truncate(self.current + 1);
        self.played.truncate(self.current);
        self.positions.push(next);
        self.played.push(mv);
        self.current += 1;
    }

    /// Return the move that led to the current position, or None at the start
    pub fn last_move(&self) -> Option<Move> {
        self.current.checked_sub(1).map(|index| self.played[index])
    }

    /// Return true if there's an earlier position to go back to
    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    /// Return true if there's an undone position to go forward to
    pub fn can_redo(&self) -> bool {
        self.current + 1 < self.positions.len()
    }

    /// Go back one position
    pub fn undo(&mut self) {
        debug_assert!(self.can_undo(), "nothing to undo");
        self.current -= 1;
    }

    /// Go forward one undone position
    pub fn redo(&mut self) {
        debug_assert!(self.can_redo(), "nothing to redo");
        self.current += 1;
    }

    /// Return how the game ended, or None if it's still going
    pub fn outcome(&self) -> Option<GameOutcome> {
        let history: Vec<PositionHash> = self.positions[..self.current]
            .iter()
            .map(|board| board.hash())
            .collect();

        game_outcome(&self.current_position(), &self.moves(), &history)
    }
}

impl Default for GameSession {
    /// Start a game from the starting position
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::outcome::DrawReason;
    use crate::game::side::Side;

    // Two kings stepping out and back, which returns to the starting position every 4 plies
    const KING_SHUFFLE: &str = "B:WK29:BK1";
    const SHUFFLE_CYCLE: [&str; 4] = ["1-5", "29-25", "5-1", "25-29"];

    /// Play the legal move with the given notation, like 9-13
    fn play(session: &mut GameSession, notation: &str) {
        let mv = session
            .moves()
            .into_iter()
            .find(|mv| mv.to_string() == notation)
            .unwrap_or_else(|| panic!("{notation} is not legal"));
        session.play(mv);
    }

    /// Return the current position as FEN, to compare positions easily
    fn fen(session: &GameSession) -> String {
        session.current_position().to_fen()
    }

    #[test]
    fn new_game_starts_at_the_start_position() {
        let session = GameSession::new();

        assert_eq!(session.current_position(), Board::start_position());
        assert!(!session.can_undo());
        assert!(!session.can_redo());
        assert_eq!(session.outcome(), None);
    }

    #[test]
    fn playing_a_move_changes_the_position() {
        let mut session = GameSession::new();
        play(&mut session, "9-13");

        assert_eq!(
            fen(&session),
            "W:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,10,11,12,13"
        );
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn undo_and_redo_step_through_the_history() {
        let mut session = GameSession::new();
        let start = fen(&session);
        play(&mut session, "9-13");
        let after_first = fen(&session);
        play(&mut session, "22-18");
        let after_second = fen(&session);

        // Back to the start, one position at a time
        session.undo();
        assert_eq!(fen(&session), after_first);
        session.undo();
        assert_eq!(fen(&session), start);
        assert!(!session.can_undo());

        // And forward again
        session.redo();
        assert_eq!(fen(&session), after_first);
        session.redo();
        assert_eq!(fen(&session), after_second);
        assert!(!session.can_redo());
    }

    #[test]
    fn playing_after_an_undo_drops_the_redo_positions() {
        let mut session = GameSession::new();
        play(&mut session, "9-13");
        play(&mut session, "22-18");

        // Undo White's move and play a different one instead
        session.undo();
        play(&mut session, "21-17");

        assert!(!session.can_redo());
        session.undo();
        session.redo();
        assert!(fen(&session).contains("W17,"));
    }

    #[test]
    fn last_move_follows_undo_and_redo() {
        let mut session = GameSession::new();
        assert_eq!(session.last_move(), None);

        play(&mut session, "9-13");
        play(&mut session, "22-18");
        assert_eq!(session.last_move().unwrap().to_string(), "22-18");

        session.undo();
        assert_eq!(session.last_move().unwrap().to_string(), "9-13");
        session.undo();
        assert_eq!(session.last_move(), None);

        // A new branch replaces the undone moves
        session.redo();
        play(&mut session, "21-17");
        assert_eq!(session.last_move().unwrap().to_string(), "21-17");
    }

    #[test]
    fn games_can_start_from_a_fen() {
        let session = GameSession::from_fen(KING_SHUFFLE).unwrap();
        assert_eq!(fen(&session), KING_SHUFFLE);

        assert!(GameSession::from_fen("not a position").is_err());
    }

    #[test]
    fn outcome_follows_the_game_history() {
        let mut session = GameSession::from_fen(KING_SHUFFLE).unwrap();

        // Two shuffles bring the starting position back for the third time
        for _ in 0..2 {
            for notation in SHUFFLE_CYCLE {
                play(&mut session, notation);
            }
        }
        assert_eq!(
            session.outcome(),
            Some(GameOutcome::Draw(DrawReason::ThreefoldRepetition))
        );

        // Undoing one move goes back to a position that has only appeared twice
        session.undo();
        assert_eq!(session.outcome(), None);
    }

    #[test]
    fn side_with_no_pieces_has_lost() {
        let session = GameSession::from_fen("B:W21:B").unwrap();
        assert_eq!(session.outcome(), Some(GameOutcome::Win(Side::White)));
    }
}
