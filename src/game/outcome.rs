use super::board::Board;
use super::hash::PositionHash;
use super::moves::MoveList;
use super::side::Side;

/// Plies without a man move or capture before the game is drawn (40 moves by each side)
pub const FORTY_MOVE_RULE_PLIES: u8 = 80;

/// Fewest plies since progress that can hold a threefold repetition
/// (the position, then two returns to it that take 4 plies each)
pub const MIN_PLIES_FOR_THREEFOLD: usize = 8;

/// Earlier appearances of a position needed for a threefold repetition
/// (the current appearance makes the third)
pub const NUM_POSITION_MATCHES_FOR_THREEFOLD: u8 = 2;

/// Why a game ended in a draw
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawReason {
    ThreefoldRepetition,
    FortyMoveRule,
}

/// How a finished game ended
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOutcome {
    Win(Side),
    Draw(DrawReason),
}

/// Return true if the current position has appeared twice before
fn is_threefold_repetition(
    current_hash: PositionHash,
    plies_since_progress: u8,
    history: &[PositionHash],
) -> bool {
    let history_plies = history.len();
    let plies_since_progress = plies_since_progress as usize;

    // The history must reach back at least to the last man move or capture
    debug_assert!(
        history_plies >= plies_since_progress,
        "history is shorter than the plies since progress"
    );

    // Too few reversible plies for the position to have come back twice yet
    if plies_since_progress < MIN_PLIES_FOR_THREEFOLD {
        return false;
    }

    // A man move or capture can never be undone, so only positions since the
    // last one can repeat
    //     history of 100, 20 plies since progress -> check the last 20
    let start = history_plies - plies_since_progress;
    let window = &history[start..];

    // Walk back through positions with the same side to move as now:
    //     1. Reverse, so the most recent position comes first
    //     2. Skip one, since the previous position had the other side to move
    //     3. Step by two from there, since sides alternate every ply
    let mut count = 0;
    for &hash in window.iter().rev().skip(1).step_by(2) {
        if hash == current_hash {
            count += 1;
        }

        // Two earlier appearances plus this one make three
        if count == NUM_POSITION_MATCHES_FOR_THREEFOLD {
            return true;
        }
    }

    false
}

/// Return how the game ended, or None if it's still going
pub fn game_outcome(
    board: &Board,
    moves: &MoveList,
    history: &[PositionHash],
) -> Option<GameOutcome> {
    // A side with no legal moves loses, whether its pieces are all captured or
    // all blocked. This ends the game before any draw rule applies
    if moves.is_empty() {
        let winner = board.side_to_move().opposite();
        return Some(GameOutcome::Win(winner));
    }

    let plies_since_progress = board.plies_since_progress();
    debug_assert!(
        plies_since_progress <= FORTY_MOVE_RULE_PLIES,
        "moves were played after the 40-move rule ended the game"
    );

    // 40 moves by each side without a man move or capture is a draw.
    if plies_since_progress == FORTY_MOVE_RULE_PLIES {
        return Some(GameOutcome::Draw(DrawReason::FortyMoveRule));
    }

    // The same position with the same side to move for the third time is a draw
    if is_threefold_repetition(board.hash(), plies_since_progress, history) {
        return Some(GameOutcome::Draw(DrawReason::ThreefoldRepetition));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::fen::START_FEN;
    use crate::game::generate_moves;

    // Two kings stepping out and back, which returns to the starting position every 4 plies
    const KING_SHUFFLE: &str = "B:WK29:BK1";
    const SHUFFLE_CYCLE: [&str; 4] = ["1-5", "29-25", "5-1", "25-29"];

    /// Play moves from a position, keeping the hash of every position before the last
    fn play_moves(fen: &str, notations: &[&str]) -> (Board, Vec<PositionHash>) {
        let mut board = Board::from_fen(fen).unwrap();
        let mut history = Vec::new();

        for &notation in notations {
            let mv = generate_moves(&board)
                .into_iter()
                .find(|mv| mv.to_string() == notation)
                .unwrap_or_else(|| panic!("{notation} is not legal in {}", board.to_fen()));

            history.push(board.hash());
            board.make_move(mv);
        }

        (board, history)
    }

    /// Return the outcome after playing the moves from the given position
    fn outcome_after(fen: &str, notations: &[&str]) -> Option<GameOutcome> {
        let (board, history) = play_moves(fen, notations);
        game_outcome(&board, &generate_moves(&board), &history)
    }

    /// Repeat the king shuffle the given number of times
    fn shuffles(cycles: usize) -> Vec<&'static str> {
        SHUFFLE_CYCLE.repeat(cycles)
    }

    #[test]
    fn start_position_is_still_going() {
        assert_eq!(outcome_after(START_FEN, &[]), None);
    }

    #[test]
    fn side_with_no_pieces_loses() {
        assert_eq!(
            outcome_after("B:W21:B", &[]),
            Some(GameOutcome::Win(Side::White))
        );
    }

    #[test]
    fn side_with_every_piece_blocked_loses() {
        // The man on 5 can't step onto 9, and can't jump it because 14 is taken
        assert_eq!(
            outcome_after("B:W9,14:B5", &[]),
            Some(GameOutcome::Win(Side::White))
        );
    }

    #[test]
    fn second_occurrence_is_not_a_draw_yet() {
        assert_eq!(outcome_after(KING_SHUFFLE, &shuffles(1)), None);
    }

    #[test]
    fn third_occurrence_is_a_repetition_draw() {
        assert_eq!(
            outcome_after(KING_SHUFFLE, &shuffles(2)),
            Some(GameOutcome::Draw(DrawReason::ThreefoldRepetition))
        );
    }

    #[test]
    fn forty_moves_without_progress_is_a_draw() {
        // 20 cycles of 4 plies is 80 plies, and the 40-move rule is checked
        // before repetition
        assert_eq!(
            outcome_after(KING_SHUFFLE, &shuffles(20)),
            Some(GameOutcome::Draw(DrawReason::FortyMoveRule))
        );
    }

    #[test]
    fn repetition_only_looks_back_to_the_last_progress() {
        // Fake hashes, with the current position (1) at indices 0 and 2
        let history = [1, 20, 1, 21, 22, 23, 24, 25, 26, 27];

        // With 10 plies since progress both earlier appearances count
        assert!(is_threefold_repetition(1, 10, &history));

        // With 8, index 0 was before the last progress and can't count
        assert!(!is_threefold_repetition(1, 8, &history));
    }

    #[test]
    fn positions_with_the_other_side_to_move_never_count() {
        // The current position's hash sits only at odd distances back, where the
        // other side was to move
        let history = [20, 1, 21, 1, 22, 1, 23, 1];
        assert!(!is_threefold_repetition(1, 8, &history));
    }
}
