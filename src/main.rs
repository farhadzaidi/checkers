mod game;

use game::Board;

fn main() {
    let board = Board::new();
    println!("{board}");
}
