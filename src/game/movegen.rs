use super::bitboard::{
    Bitboard, EMPTY_BITBOARD, Square, is_empty, pop_lsb, shift, square_mask, to_square,
};
use super::board::Board;
use super::direction::{Direction, DirectionPair, KING_DIRECTIONS, forward};
use super::moves::{Move, MoveList, MovePath};

/// Snapshot of one position for a single move generation
struct MoveGenerator {
    us: Bitboard,
    them: Bitboard,
    our_kings: Bitboard,
    empty: Bitboard,
    forward_directions: DirectionPair,
    backward_directions: DirectionPair,
    promotion_row: Bitboard,
}

/// What stays the same while following one jumper's paths
struct JumpContext<'a> {
    // Square the jumper started on, which every move from this search starts from
    origin: Square,
    // Borrowed from the generator or the king directions, so the search can't outlive them
    directions: &'a [Direction],
    // Landing here ends the move: the promotion row for men, nothing for kings
    stop_row: Bitboard,
    // Empty squares, counting the jumper's own starting square
    empty: Bitboard,
}

impl JumpContext<'_> {
    /// Return the piece jumped and the square landed on, jumping from the current square in this
    /// direction, or None if there's nothing jumpable next to it or the landing is blocked
    fn jump(
        &self,
        current: Bitboard,
        jumpable: Bitboard,
        direction: Direction,
    ) -> Option<(Bitboard, Bitboard)> {
        // Step once and keep a piece that may be jumped
        let capture = shift(current, direction) & jumpable;
        if is_empty(capture) {
            return None;
        }

        // Step once more and keep the landing square if it's empty
        let landing = shift(capture, direction) & self.empty;
        if is_empty(landing) {
            return None;
        }

        Some((capture, landing))
    }
}

impl MoveGenerator {
    /// Capture the bitboards the generator needs from the board
    fn new(board: &Board) -> Self {
        debug_assert!(board.is_valid(), "generating moves for an invalid board");

        let side = board.side_to_move();
        Self {
            us: board.pieces(side),
            them: board.pieces(side.opposite()),
            our_kings: board.kings(side),
            empty: board.empty(),
            forward_directions: forward(side),
            backward_directions: forward(side.opposite()),
            promotion_row: side.promotion_row(),
        }
    }

    /// Return every one of our pieces that has at least one jump available
    fn find_jumpers(&self) -> Bitboard {
        let mut result = EMPTY_BITBOARD;

        // Work backwards from the landing squares, one direction at a time:
        //     1. Start from every empty square (a possible landing spot)
        //     2. Step once against the jump direction and keep enemy pieces
        //         (an enemy with an empty square behind it)
        //     3. Step once more and keep our movers
        //         (a mover with that enemy in front of it)
        // What's left are the movers that can jump in that direction.
        let jumpers = |movers: Bitboard, direction: Direction| {
            let back = direction.opposite();
            let potential_captures = shift(self.empty, back) & self.them;
            shift(potential_captures, back) & movers
        };

        // Forward directions for all our pieces (men + kings)
        for direction in self.forward_directions {
            result |= jumpers(self.us, direction);
        }

        // Backward directions for kings only
        for direction in self.backward_directions {
            result |= jumpers(self.our_kings, direction);
        }

        debug_assert!(
            is_empty(result & !self.us),
            "found a jumper that isn't ours"
        );

        result
    }

    /// Push every complete jump sequence for each of the given jumpers
    fn generate_captures(&self, jumpers: Bitboard, moves: &mut MoveList) {
        let mut remaining = jumpers;
        while !is_empty(remaining) {
            let jumper = pop_lsb(&mut remaining);
            let context = self.jump_context(jumper);
            self.jump_search(&context, square_mask(jumper), EMPTY_BITBOARD, moves);
        }
    }

    /// Set up a jump search for the piece on the given square
    fn jump_context(&self, jumper: Square) -> JumpContext<'_> {
        let jumper_mask = square_mask(jumper);

        // Men only jump forward and stop on the promotion row; kings never stop early
        let is_man = is_empty(jumper_mask & self.our_kings);
        let (directions, stop_row): (&[Direction], Bitboard) = if is_man {
            (&self.forward_directions, self.promotion_row)
        } else {
            (&KING_DIRECTIONS, EMPTY_BITBOARD)
        };

        JumpContext {
            origin: jumper,
            directions,
            stop_row,
            empty: jumper_mask | self.empty,
        }
    }

    /// Push one move for every complete jump path from the current square
    fn jump_search(
        &self,
        context: &JumpContext,
        current: Bitboard,
        captured: Bitboard,
        moves: &mut MoveList,
    ) {
        // Depth-first search over jump paths. For each allowed direction:
        //     1. Try to jump an enemy piece not yet captured in this sequence
        //         (nothing to jump or a blocked landing means try the next direction)
        //     2. Landing on the stop row ends the move (a man reaching its promotion
        //         row is crowned): push the move here instead of recursing
        //     3. Otherwise recurse from the landing square with the jumped piece
        //         added to the captures
        // Each branch gets its own copy of the current square and captures, so paths
        // never affect each other.
        //
        // If no direction produced a jump, this square is where the path ends:
        // push one move from the origin to here with every capture along the way.

        // Any enemy piece can be jumped, as long as it wasn't already jumped on this path
        let jumpable = self.them & !captured;

        let mut can_continue_jumping = false;
        for &direction in context.directions {
            let Some((capture, landing)) = context.jump(current, jumpable, direction) else {
                continue;
            };

            // Found a capture/landing spot
            can_continue_jumping = true;

            // Stop here if a man lands on a promotion row
            if !is_empty(landing & context.stop_row) {
                let mv = Move::new(context.origin, to_square(landing), captured | capture);
                moves.push(mv);
                continue;
            }

            self.jump_search(context, landing, captured | capture, moves);
        }

        // We hit a dead end so we can push the move here
        if !can_continue_jumping {
            // Every jumper has at least one jump, so a finished path always captured something
            debug_assert!(!is_empty(captured), "jump path ended without a capture");

            let mv = Move::new(context.origin, to_square(current), captured);
            moves.push(mv);
        }
    }

    /// Follow the given capture from the current square, pushing each landing square onto the path.
    /// Return true once the path takes every piece the move captures and ends on its final square
    fn find_path(
        &self,
        context: &JumpContext,
        current: Bitboard,
        captured: Bitboard,
        mv: Move,
        path: &mut MovePath,
    ) -> bool {
        // Every piece taken: this is the route only if it ended where the move ends
        if captured == mv.captures() {
            return current == square_mask(mv.to());
        }

        // Same steps as jump_search, but only over pieces this move captures. A direction
        // that leads to a dead end is taken back off the path, and the next one is tried
        //     6x22 capturing 10 and 18 -> try over 9 (not captured, skip), then over 10 to 15,
        //     then over 18 to 22 -> path [15, 22]
        let jumpable = mv.captures() & !captured;
        for &direction in context.directions {
            let Some((capture, landing)) = context.jump(current, jumpable, direction) else {
                continue;
            };

            path.push(to_square(landing));
            if self.find_path(context, landing, captured | capture, mv, path) {
                return true;
            }
            path.pop();
        }

        false
    }

    /// Push every simple one-step move (no captures)
    fn generate_quiet_moves(&self, moves: &mut MoveList) {
        let mut generate = |movers: Bitboard, direction: Direction| {
            let mut targets = shift(movers, direction) & self.empty;
            while !is_empty(targets) {
                let to = pop_lsb(&mut targets);
                let from_mask = shift(square_mask(to), direction.opposite());
                let from = to_square(from_mask);
                let mv = Move::new(from, to, EMPTY_BITBOARD);
                moves.push(mv);
            }
        };

        for direction in self.forward_directions {
            generate(self.us, direction);
        }

        for direction in self.backward_directions {
            generate(self.our_kings, direction);
        }
    }
}

/// Return the squares a legal move lands on in order, ending on its final square
pub fn move_path(board: &Board, mv: Move) -> MovePath {
    // A simple step lands once, on its destination
    let mut path = MovePath::new();
    if !mv.is_capture() {
        path.push(mv.to());
        return path;
    }

    // A capture is followed hop by hop from its starting square
    let generator = MoveGenerator::new(board);
    let context = generator.jump_context(mv.from());
    let found = generator.find_path(
        &context,
        square_mask(mv.from()),
        EMPTY_BITBOARD,
        mv,
        &mut path,
    );
    debug_assert!(found, "{mv} is not a legal capture in {}", board.to_fen());

    path
}

/// Return every legal move for the side to move
pub fn generate_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    let generator = MoveGenerator::new(board);

    let jumpers = generator.find_jumpers();
    if !is_empty(jumpers) {
        generator.generate_captures(jumpers, &mut moves);
    } else {
        generator.generate_quiet_moves(&mut moves);
    }

    // Captures are mandatory, so either every move captures or none do
    debug_assert!(
        moves
            .iter()
            .all(|mv| is_empty(mv.captures()) == is_empty(jumpers)),
        "mixed captures and quiet moves in one move list"
    );

    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Return the legal moves in a position as sorted notation, like ["9-13", "9-14"]
    fn legal_moves(fen: &str) -> Vec<String> {
        let board = Board::from_fen(fen).unwrap();
        let mut moves: Vec<String> = generate_moves(&board)
            .iter()
            .map(|mv| mv.to_string())
            .collect();
        moves.sort();
        moves
    }

    /// Return the captured squares of every legal move, using notation squares 1-32
    fn captured_squares(fen: &str) -> Vec<Vec<Square>> {
        let board = Board::from_fen(fen).unwrap();
        let mut result = Vec::new();

        for mv in generate_moves(&board) {
            let squares = mv
                .captured_squares()
                .iter()
                .map(|square| square + 1)
                .collect();
            result.push(squares);
        }

        result.sort();
        result
    }

    #[test]
    fn start_position_has_seven_moves() {
        let expected = ["10-14", "10-15", "11-15", "11-16", "12-16", "9-13", "9-14"];
        assert_eq!(legal_moves(crate::game::pdn::START_FEN), expected);
    }

    #[test]
    fn white_men_move_up_the_board() {
        assert_eq!(legal_moves("W:W25:B"), ["25-21", "25-22"]);
    }

    #[test]
    fn kings_move_in_all_four_directions() {
        assert_eq!(
            legal_moves("B:W:BK18"),
            ["18-14", "18-15", "18-22", "18-23"]
        );
    }

    #[test]
    fn pieces_on_the_edges_do_not_wrap_around() {
        // 5 is on the left edge and 12 on the right edge, so each has one move
        assert_eq!(legal_moves("B:W:B5,12"), ["12-16", "5-9"]);
    }

    #[test]
    fn captures_are_mandatory() {
        // The man on 1 has quiet moves, but the capture on 9 must be taken
        assert_eq!(legal_moves("B:W14:B1,9"), ["9x18"]);
    }

    #[test]
    fn branching_jumps_are_separate_moves() {
        // After 9x18 the man can continue over 22 or over 23
        assert_eq!(legal_moves("B:W14,22,23:B9"), ["9x25", "9x27"]);
        assert_eq!(captured_squares("B:W14,22,23:B9"), [[14, 22], [14, 23]]);
    }

    #[test]
    fn crowning_ends_the_jump() {
        // Landing on 31 crowns the man, so it can't go on to jump 27 as a king
        assert_eq!(legal_moves("B:W26,27:B22"), ["22x31"]);
        assert_eq!(captured_squares("B:W26,27:B22"), [[26]]);
    }

    #[test]
    fn kings_jump_backward_and_change_direction() {
        // Up over 14 to 9, then turn and go up over 6 to 2
        assert_eq!(legal_moves("B:W6,14:BK18"), ["18x2"]);
        assert_eq!(captured_squares("B:W6,14:BK18"), [[6, 14]]);
    }

    #[test]
    fn king_can_loop_back_through_its_own_square() {
        // Four pieces around 10 let the king jump a full loop and land where it started.
        // Going around each way captures the same pieces, and both routes are counted
        assert_eq!(legal_moves("B:W14,15,22,23:BK10"), ["10x10", "10x10"]);
        assert_eq!(
            captured_squares("B:W14,15,22,23:BK10"),
            [[14, 15, 22, 23], [14, 15, 22, 23]]
        );
    }

    #[test]
    fn side_with_no_pieces_has_no_moves() {
        assert!(legal_moves("B:W21:B").is_empty());
    }

    /// Return every legal move with the squares it lands on, using notation squares 1-32
    fn paths(fen: &str) -> Vec<(String, Vec<Square>)> {
        let board = Board::from_fen(fen).unwrap();
        let mut result = Vec::new();

        for mv in generate_moves(&board) {
            let landings = move_path(&board, mv)
                .iter()
                .map(|square| square + 1)
                .collect();
            result.push((mv.to_string(), landings));
        }

        result.sort();
        result
    }

    #[test]
    fn simple_step_lands_once() {
        assert_eq!(paths("B:W:B5"), [("5-9".to_string(), vec![9])]);
    }

    #[test]
    fn two_routes_to_the_same_square_have_different_paths() {
        // Around the diamond of 9, 10, 17 and 18: down the left through 13, or the right through 15
        assert_eq!(
            paths("B:W9,10,17,18:B6"),
            [
                ("6x22".to_string(), vec![13, 22]),
                ("6x22".to_string(), vec![15, 22]),
            ]
        );
    }

    #[test]
    fn king_path_follows_its_turns() {
        // Up over 14 to 9, then over 6 to 2
        assert_eq!(paths("B:W6,14:BK18"), [("18x2".to_string(), vec![9, 2])]);
    }

    #[test]
    fn looping_king_path_ends_where_it_started() {
        // Four jumps around 10, landing back on 10
        for (notation, landings) in paths("B:W14,15,22,23:BK10") {
            assert_eq!(notation, "10x10");
            assert_eq!(landings.len(), 4);
            assert_eq!(landings.last(), Some(&10));
        }
    }
}
