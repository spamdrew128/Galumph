use crate::{
    move_generation::{
        board_rep::Board,
        perft::{test_postions, PerftTest},
    },
    search::search_manager::SearchManager,
};

pub fn run_bench() {
    let positions: Vec<PerftTest> = test_postions();

    let stopwatch = std::time::Instant::now();
    let mut nodes = 0;

    let mut search_manager = SearchManager::new();

    for pos in positions {
        let board = Board::from_fen(pos.fen);
        search_manager.update_board(&board);
        nodes += search_manager.start_bench_search(8);
    }

    let nps = (u128::from(nodes) * 1_000_000) / stopwatch.elapsed().as_micros();
    println!("{nodes} nodes {nps} nps");
}
