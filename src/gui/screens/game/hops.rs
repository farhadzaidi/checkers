use super::screen::GameScreen;
use crate::game::{Move, MoveList, MovePath, Square};
use eframe::egui::Pos2;

impl GameScreen {
    /// Return the square the selected piece stands on: its last hop in a capture, or where it started
    pub(super) fn piece_square(&self) -> Option<Square> {
        let selected = self.selected?;
        Some(self.hops.last().copied().unwrap_or(selected))
    }

    /// Return true while the selected piece is partway through a capture
    pub(super) fn is_mid_capture(&self) -> bool {
        !self.hops.is_empty()
    }

    /// Return the legal moves of the selected piece, or none when nothing is selected
    fn selected_moves(&self) -> MoveList {
        match self.selected {
            Some(from) => self.session.moves_from(from),
            None => MoveList::new(),
        }
    }

    /// Return the selected piece's moves that carry on from the hops made so far, each with its route
    pub(super) fn routes(&self) -> Vec<(Move, MovePath)> {
        self.selected_moves()
            .into_iter()
            .map(|mv| (mv, self.session.move_path(mv)))
            .filter(|(_, path)| path.starts_with(&self.hops))
            .collect()
    }

    /// Return each square the selected piece can land on next, and whether getting there captures
    pub(super) fn next_landings(&self) -> Vec<(Square, bool)> {
        let mut landings: Vec<(Square, bool)> = Vec::new();
        for (mv, path) in self.routes() {
            // Two routes can share their next hop, so mark each square once
            let next = path[self.hops.len()];
            if !landings.iter().any(|&(square, _)| square == next) {
                landings.push((next, mv.is_capture()));
            }
        }
        landings
    }

    /// Move the selected piece to the target if one of its routes goes there, playing the move once its
    /// route is complete. Return false if no route lands on the target
    pub(super) fn land(&mut self, target: Square, dropped_at: Option<Pos2>) -> bool {
        let done = self.hops.len();
        let Some(start) = self.piece_square() else {
            return false;
        };

        // The square the piece stands on is never a landing, so clicking it can deselect it. A king's loop
        // back to its own square still works hop by hop, since its last hop starts from a different square
        if target == start {
            return false;
        }

        // Each route that lands on the target, with its landings up to and including the target
        //     diamond from 6, target 13 -> only the left route, landings [13]
        //     diamond from 6, target 22 -> both routes, landings [13, 22] and [15, 22]
        let routes = self.routes();
        let reaching: Vec<(Move, &MovePath, &[Square])> = routes
            .iter()
            .filter_map(|(mv, path)| Some((*mv, path, landings_to(path, done, target)?)))
            .collect();
        let Some(&(mv, path, landings)) = reaching.first() else {
            return false;
        };

        // Routes that reach the target different ways need the player to pick the way, one hop at a time
        if reaching.iter().any(|&(_, _, other)| other != landings) {
            return true;
        }

        // The hops this click or drop covers, which the piece slides over
        let new_hops: MovePath = landings[done..].iter().copied().collect();
        if landings.len() == path.len() {
            self.play(mv, start, new_hops, dropped_at);
        } else {
            self.hops = landings.iter().copied().collect();
            self.slide_hops(start, new_hops, dropped_at);
        }
        true
    }

    /// Put the piece back where it started, dropping the hops made so far
    pub(super) fn cancel_hops(&mut self) {
        self.hops.clear();
        self.slide = None;
    }

    /// Take back the last hop of the capture in progress, leaving the piece on the hop before.
    /// A drag in progress is dropped, since the piece it picked up has moved
    pub(super) fn step_back_hop(&mut self) {
        self.hops.pop();
        self.slide = None;
        self.dragging = None;
    }
}

/// Return the route's landings from the start up to the first time it lands on the target
/// after the given number of hops, or None if it doesn't land there
fn landings_to(path: &MovePath, done: usize, target: Square) -> Option<&[Square]> {
    let index = path[done..].iter().position(|&square| square == target)?;
    Some(&path[..=done + index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{GameSession, Side};

    // Black's man on 6 can capture around a diamond of White men on 9, 10, 17 and 18,
    // landing on 22 either way. Squares below count from 0, so notation square 6 is 5
    const DIAMOND: &str = "B:W9,10,17,18:B6";

    // The same diamond with White to move first. Black's men on 1 and 2 stop White's men on 9 and 10
    // from jumping the man on 6, so White plays a quiet move with the man on 32
    const WHITE_FIRST: &str = "W:W9,10,17,18,32:B1,2,6";
    const START: Square = 5;
    const LEFT_HOP: Square = 12;
    const RIGHT_HOP: Square = 14;
    const END: Square = 21;

    /// A game in the diamond position with the man on 6 picked up
    fn diamond() -> GameScreen {
        let session = GameSession::from_fen(DIAMOND).unwrap();
        let mut screen = GameScreen::new(session, Side::Black, None);
        screen.selected = Some(START);
        screen
    }

    #[test]
    fn first_hops_of_both_routes_are_offered() {
        let mut landings: Vec<Square> = diamond()
            .next_landings()
            .iter()
            .map(|&(square, _)| square)
            .collect();
        landings.sort();
        assert_eq!(landings, [LEFT_HOP, RIGHT_HOP]);
    }

    #[test]
    fn going_straight_to_a_shared_end_waits_for_a_route() {
        // Both routes end on 22, so landing there can't pick one: nothing is played
        let mut screen = diamond();
        assert!(screen.land(END, None));
        assert!(screen.hops.is_empty());
        assert!(!screen.session.can_undo());
    }

    #[test]
    fn hopping_left_then_down_plays_the_left_route() {
        let mut screen = diamond();
        assert!(screen.land(LEFT_HOP, None));
        assert_eq!(screen.hops.as_slice(), &[LEFT_HOP]);

        // Only the left route is left, so its next hop finishes the move
        assert_eq!(screen.next_landings(), [(END, true)]);
        assert!(screen.land(END, None));

        // The left route took the men on 9 and 17
        let played = screen.session.last_move().unwrap();
        assert_eq!(played.captured_squares().as_slice(), &[8, 16]);
        assert!(screen.hops.is_empty());
    }

    #[test]
    fn a_square_off_every_route_is_refused() {
        let mut screen = diamond();
        assert!(!screen.land(LEFT_HOP + 1, None));
        assert!(screen.hops.is_empty());
    }

    /// Play the legal move with the given notation, like 32-28
    fn play(session: &mut GameSession, notation: &str) -> Move {
        let mv = Move::from_notation(&session.current_position(), notation).unwrap();
        session.play(mv);
        mv
    }

    #[test]
    fn undo_mid_capture_takes_back_a_hop_not_a_move() {
        // White moves first, so there's a played move that undo must leave alone
        let mut session = GameSession::from_fen(WHITE_FIRST).unwrap();
        let white_move = play(&mut session, "32-28");

        let mut screen = GameScreen::new(session, Side::Black, None);
        screen.selected = Some(START);
        screen.land(LEFT_HOP, None);
        assert!(screen.can_undo());

        // Undo steps the piece back off its hop, and White's move stays played
        screen.undo();
        assert!(screen.hops.is_empty());
        assert_eq!(screen.session.last_move(), Some(white_move));
    }

    #[test]
    fn redo_waits_until_the_capture_is_done() {
        // Black's capture is played and undone, so it can be redone
        let mut session = GameSession::from_fen(WHITE_FIRST).unwrap();
        play(&mut session, "32-28");
        play(&mut session, "6x13x22");
        session.undo();

        let mut screen = GameScreen::new(session, Side::Black, None);
        assert!(screen.can_redo());

        // Starting a capture by hand would be thrown away by a redo, so redo waits
        screen.selected = Some(START);
        screen.land(LEFT_HOP, None);
        assert!(!screen.can_redo());
    }

    #[test]
    fn cancelling_puts_the_piece_back() {
        let mut screen = diamond();
        screen.land(RIGHT_HOP, None);
        screen.cancel_hops();
        assert!(screen.hops.is_empty());
        assert_eq!(screen.piece_square(), Some(START));
    }
}
