use super::bitboard::{Bitboard, Square, is_empty, pop_lsb};
use super::piece::Piece;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::sync::LazyLock;

// Position hashing (Zobrist hashing) gives every position a 64-bit fingerprint, so repeated
// positions can be found quickly:
//     1. Every piece type on every square gets its own random key
//     2. A position's hash is all of its pieces' keys XORed together, plus
//         one more key when White is to move
//     3. XOR undoes itself, so moving a piece just XORs its old key out and
//         its new key in, instead of recomputing the whole hash

/// A 64-bit fingerprint that identifies a position
pub type PositionHash = u64;

const NUM_PIECE_TYPES: usize = 4;
const NUM_SQUARES: usize = 32;

/// Fixed seed, so every run uses the same keys
const KEY_SEED: u64 = 0x00C0_DEC0_FFEE_1234;

/// Every random key the hash uses
struct HashKeys {
    piece_square: [[PositionHash; NUM_SQUARES]; NUM_PIECE_TYPES],
    white_to_move: PositionHash,
}

/// The keys, generated the first time any of them is used
static KEYS: LazyLock<HashKeys> = LazyLock::new(generate_keys);

/// Return the combined key for the given piece on every square in the mask
pub fn pieces_key(piece: Piece, mask: Bitboard) -> PositionHash {
    let mut key = 0;

    let mut remaining = mask;
    while !is_empty(remaining) {
        let square = pop_lsb(&mut remaining);
        key ^= piece_square_key(piece, square);
    }

    key
}

/// Return the key for the given piece on the given square
fn piece_square_key(piece: Piece, square: Square) -> PositionHash {
    KEYS.piece_square[piece as usize][square as usize]
}

/// Return the key mixed into the hash when it's White's turn
pub fn white_to_move_key() -> PositionHash {
    KEYS.white_to_move
}

/// Fill every key with a random number from a seeded generator
fn generate_keys() -> HashKeys {
    let mut rng = StdRng::seed_from_u64(KEY_SEED);

    let mut piece_square = [[0; NUM_SQUARES]; NUM_PIECE_TYPES];
    for piece_keys in &mut piece_square {
        for key in piece_keys {
            *key = rng.random();
        }
    }

    HashKeys {
        piece_square,
        white_to_move: rng.random(),
    }
}
