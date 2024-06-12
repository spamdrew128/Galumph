use crate::{
    move_generation::{
        board_rep::Board,
        perft::{test_postions, PerftTest},
    },
    search::{search_manager::SearchManager, zobrist_stack::ZobristStack},
};

pub fn run_bench() {
    let positions: Vec<PerftTest> = test_postions();

    let stopwatch = std::time::Instant::now();
    let mut nodes = 0;

    let mut search_manager = SearchManager::new();

    for pos in positions {
        let board = Board::from_fen(pos.fen);
        let zobrist_stack = ZobristStack::new(&board);

        search_manager.update_state(&board, &zobrist_stack);
        nodes += search_manager.start_bench_search(7);
    }

    let nps = (u128::from(nodes) * 1_000_000) / stopwatch.elapsed().as_micros();
    println!("{nodes} nodes {nps} nps");
}
