use checkers::game::Board;
use checkers::perft::perft;

// Published perft counts from the starting position, from Aart Bik's checkers
// perft results (also OEIS A133046). Index 0 is depth 1. These count duplicate
// king captures separately, the same as our move generator
const START_POSITION_PERFT: [u64; 14] = [
    7,
    49,
    302,
    1_469,
    7_361,
    36_768,
    179_740,
    845_931,
    3_963_680,
    18_391_564,
    85_242_128,
    388_623_673,
    1_766_623_630,
    7_978_439_499,
];

/// Check the start position's perft count at every depth in the range
fn check_start_position(depths: std::ops::RangeInclusive<u32>) {
    let board = Board::start_position();

    for depth in depths {
        let expected = START_POSITION_PERFT[depth as usize - 1];
        assert_eq!(
            perft(&board, depth),
            expected,
            "perft({depth}) from the start"
        );
    }
}

#[test]
fn start_position_up_to_depth_10() {
    check_start_position(1..=10);
}

#[test]
#[ignore = "slow, run with: cargo test --release -- --ignored"]
fn start_position_depth_11_to_14() {
    check_start_position(11..=14);
}
