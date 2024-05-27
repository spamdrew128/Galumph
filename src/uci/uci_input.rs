use crate::{
    movegen::{
        board_rep::{Board, Color, START_FEN},
        chess_move::Move,
    },
    search::search_manager::{Depth, Milliseconds, Nodes},
};

#[derive(Debug, Default, PartialEq)]
pub enum UciCommand {
    #[default]
    None,
    Uci,
    IsReady,
    UciNewGame,
    Position(Board),
    Go(Vec<GoArg>),
    Stop,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum GoArg {
    Unsupported,
    Time(Color, Milliseconds),
    Inc(Color, Milliseconds),
    MoveTime(Milliseconds),
    Nodes(Nodes),
    Depth(Depth),
    MovesToGo(u32),
    Infinite,
}

fn kill_program() {
    std::process::exit(0);
}

fn get_stdin() -> String {
    let mut buffer = String::new();
    let bytes_read = std::io::stdin()
        .read_line(&mut buffer)
        .expect("Stdio Failure");

    if bytes_read == 0 {
        kill_program();
    }

    buffer
}

fn expect_str(input: Option<&str>) -> Result<&str, ()> {
    if let Some(s) = input {
        Ok(s)
    } else {
        Err(())
    }
}

macro_rules! parse_nonzero {
    ($tokens:ident, $t:ty) => {{
        let v = $tokens.next().unwrap().parse::<$t>().unwrap_or(0);
        if v == 0 {
            Err(())
        } else {
            Ok(v)
        }
    }};
}

impl UciCommand {
    fn interpret_stdin(stdin: &str) -> Result<Self, ()> {
        let mut res = Self::default();

        let mut tokens = stdin.split_whitespace();

        let first = expect_str(tokens.next())?;
        match first {
            "stop" => res = UciCommand::Stop,
            "quit" => res = UciCommand::Quit,
            "uci" => res = UciCommand::Uci,
            "isready" => res = UciCommand::IsReady,
            "ucinewgame" => res = UciCommand::UciNewGame,
            "position" => {
                let fen_type = expect_str(tokens.next())?;
                let fen = match fen_type {
                    "startpos" => START_FEN.to_owned(),
                    "fen" => {
                        format!(
                            "{} {} {} {} {} {}",
                            expect_str(tokens.next())?,
                            expect_str(tokens.next())?,
                            expect_str(tokens.next())?,
                            expect_str(tokens.next())?,
                            expect_str(tokens.next())?,
                            expect_str(tokens.next())?
                        )
                    }
                    _ => return Err(()),
                };
                let mut board = Board::from_fen(&fen);

                while let Some(s) = tokens.next() {
                    if let Some(mv) = Move::from_str(s, &board) {
                        board.try_play_move(mv);
                    }
                }
                res = UciCommand::Position(board);
            }
            "go" => {
                let mut arglist = vec![];
                while let Some(arg) = tokens.next() {
                    let next_arg = match arg {
                        "wtime" => GoArg::Time(Color::White, parse_nonzero!(tokens, Milliseconds)?),
                        "btime" => GoArg::Time(Color::Black, parse_nonzero!(tokens, Milliseconds)?),
                        "winc" => GoArg::Inc(Color::White, parse_nonzero!(tokens, Milliseconds)?),
                        "binc" => GoArg::Inc(Color::Black, parse_nonzero!(tokens, Milliseconds)?),
                        "movetime" => GoArg::MoveTime(parse_nonzero!(tokens, Milliseconds)?),
                        "movestogo" => GoArg::MovesToGo(parse_nonzero!(tokens, u32)?),
                        "depth" => GoArg::Depth(parse_nonzero!(tokens, Depth)?),
                        "nodes" => GoArg::Nodes(parse_nonzero!(tokens, Nodes)?),
                        "infinite" => GoArg::Infinite,
                        _ => GoArg::Unsupported,
                    };

                    arglist.push(next_arg);
                }
            }
            _ => (),
        };

        Ok(res)
    }

    pub fn recieve_valid() -> Self {
        loop {
            let stdin = get_stdin();

            if let Ok(ucicommand) = Self::interpret_stdin(&stdin) {
                return ucicommand;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        movegen::{board_rep::Board, chess_move::Move},
        uci::uci_input::UciCommand,
    };

    #[test]
    fn pos_test() {
        let uci = "position fen rnbq1bnr/ppppkppp/8/1B2p3/4P3/8/PPPP1PPP/RNBQK1NR w KQ - 2 3 moves h2h4 e7f6";
        let mut expected =
            Board::from_fen("rnbq1bnr/ppppkppp/8/1B2p3/4P3/8/PPPP1PPP/RNBQK1NR w KQ - 2 3");
        expected.try_play_move(Move::from_str("h2h4", &expected).unwrap());
        expected.try_play_move(Move::from_str("e7f6", &expected).unwrap());

        assert_eq!(
            UciCommand::Position(expected),
            UciCommand::interpret_stdin(&uci).unwrap()
        );
    }
}
