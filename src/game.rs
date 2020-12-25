use rand::{seq::SliceRandom, Rng};
use std::{collections::HashMap, io};

use Color::*;
use Density::*;
use Height::*;
use Shape::*;

macro_rules! check_for_piece {
    ($e:expr) => {
        if let Some(p) = $e {
            p
        } else {
            return false;
        }
    };
}

pub fn _main() {
    let mut pieces = Vec::with_capacity(16);

    for height in &[Tall, Short] {
        for color in &[Dark, Light] {
            for density in &[Solid, Hollow] {
                for shape in &[Square, Round] {
                    pieces.push(Piece {
                        height: height.clone(),
                        color: color.clone(),
                        density: density.clone(),
                        shape: shape.clone(),
                    });
                }
            }
        }
    }

    let mut game = Game {
        board: empty_board(),
        pieces: pieces,
        last_play: Play::Placed(Player::Human),
        rng: rand::thread_rng(),
    };

    while !game.is_over() {
        game.tick();
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub(crate) pieces: Vec<Piece>,
    pub(crate) board: Board,
    pub(crate) last_play: Play,
    rng: rand::rngs::ThreadRng,
}

impl Default for Game {
    fn default() -> Self {
        let mut pieces = Vec::with_capacity(16);

        for height in &[Tall, Short] {
            for color in &[Dark, Light] {
                for density in &[Solid, Hollow] {
                    for shape in &[Square, Round] {
                        pieces.push(Piece {
                            height: height.clone(),
                            color: color.clone(),
                            density: density.clone(),
                            shape: shape.clone(),
                        });
                    }
                }
            }
        }

        let board = empty_board();
        Self {
            board: board,
            pieces: pieces,
            last_play: Play::Placed(Player::Human),
            rng: rand::thread_rng(),
        }
    }
}

impl Game {
    pub(crate) fn try_stage_piece(&mut self, new_idx: usize) {
        if let Play::Placed(Player::Human) = self.last_play {
            let piece = self.pieces.remove(new_idx);
            self.last_play = Play::Staged(Player::Human, piece);
        }
    }

    pub(crate) fn try_place_piece(&mut self, square: Coord) {
        if self.board[square.0][square.1].is_some() {
            seed::log!("square already occupied");
            return;
        }

        let play = std::mem::replace(&mut self.last_play, Play::Transitioning);
        if let Play::Staged(Player::Machine, piece) = play {
            self.board[square.0][square.1] = Some(piece);

            if self.is_win(&square) {
                self.last_play = Play::Finished(Resolution::Win(Player::Human))
            } else {
                self.last_play = Play::Placed(Player::Human)
            }
        } else {
            seed::log!("Invalid state");
            self.last_play = play;
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.pieces.is_empty() {
            println!("Game ended in draw");
            self.last_play = Play::Finished(Resolution::Draw);
            return;
        }

        let play = std::mem::replace(&mut self.last_play, Play::Transitioning);
        match play {
            // Human must stage a piece
            Play::Placed(Player::Human) => {
                let new_idx = self.stage_prompt();
                let piece = self.pieces.remove(new_idx);
                self.last_play = Play::Staged(Player::Human, piece);
            }
            // Machine must place staged piece
            Play::Staged(Player::Human, piece) => {
                let scores = self.monte_placement_scores(&piece);
                let (square, score) = scores.iter().max_by_key(|(_, score)| *score).unwrap();
                println!(
                    "Playing piece in position {:?} with score: {}",
                    square, score
                );

                self.board[square.0][square.1] = Some(piece);

                if self.is_win(square) {
                    println!("You lose!");
                    self.last_play = Play::Finished(Resolution::Win(Player::Machine))
                } else {
                    self.last_play = Play::Placed(Player::Machine)
                }
                print_board(&self.board);
            }
            // Machine must stage a piece
            Play::Placed(Player::Machine) => {
                let scores = self.monte_stage_scores();
                let (idx, score) = scores
                    .into_iter()
                    .enumerate()
                    .max_by_key(|(_idx, score)| *score)
                    .unwrap();
                let piece = self.pieces.remove(idx);
                println!("Staging piece with score: {}", score);
                self.last_play = Play::Staged(Player::Machine, piece);
            }
            // Human must place staged piece
            Play::Staged(Player::Machine, piece) => {
                let square = self.placement_prompt(&piece);
                self.board[square.0][square.1] = Some(piece);

                if self.is_win(&square) {
                    println!("You win!");
                    self.last_play = Play::Finished(Resolution::Win(Player::Human))
                } else {
                    self.last_play = Play::Placed(Player::Human)
                }

                print_board(&self.board);
            }
            Play::Transitioning | Play::Finished(_) => unreachable!("Tick invariant not upheld"),
        }
    }

    fn stage_prompt(&self) -> usize {
        println!("Remaining pieces");
        for (idx, piece) in self.pieces.iter().enumerate() {
            println!("{}: {}", idx, piece.display());
        }

        let mut input_text = String::new();
        io::stdin()
            .read_line(&mut input_text)
            .expect("Failed to read from STDIN");

        match input_text.trim().parse::<usize>() {
            Ok(n) => {
                if n < self.pieces.len() {
                    n
                } else {
                    println!("Number out of range");
                    self.stage_prompt()
                }
            }
            _ => {
                println!("Unexpected input");
                self.stage_prompt()
            }
        }
    }

    fn placement_prompt(&self, staged: &Piece) -> Coord {
        println!(
            "Machine staged piece {}. Choose square on which to place it",
            staged.display()
        );
        let mut input_text = String::new();
        io::stdin()
            .read_line(&mut input_text)
            .expect("Failed to read from STDIN");

        match input_text.trim().parse::<usize>() {
            Ok(n) => {
                let row = n / 10;
                let col = n % 10;
                if row < BOARD_SIZE && col < BOARD_SIZE {
                    if self.board[row][col].is_none() {
                        (row, col)
                    } else {
                        println!("Square already has a piece");
                        self.placement_prompt(staged)
                    }
                } else {
                    println!("Number out of range");
                    self.placement_prompt(staged)
                }
            }
            _ => {
                println!("Unexpected input");
                self.placement_prompt(staged)
            }
        }
    }

    fn empty_squares(&self) -> Vec<Coord> {
        self.board
            .iter()
            .enumerate()
            .flat_map(|(row_idx, row)| {
                row.iter().enumerate().filter_map(move |(col_idx, cell)| {
                    if cell.is_none() {
                        Some((row_idx, col_idx))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub(crate) fn is_over(&self) -> bool {
        matches!(self.last_play, Play::Finished(_))
    }

    fn is_win(&self, square: &Coord) -> bool {
        // let piece = self.board[square.0][square.1].expect("WinCheck square must contain a piece");
        self.winning_row(square)
            || self.winning_col(square)
            || self.winning_upward_diagonal(square)
            || self.winning_downward_diagonal(square)
    }

    fn winning_row(&self, square: &Coord) -> bool {
        let row_idx = square.0;

        let row = [
            check_for_piece!(self.board[row_idx][0].as_ref()),
            check_for_piece!(self.board[row_idx][1].as_ref()),
            check_for_piece!(self.board[row_idx][2].as_ref()),
            check_for_piece!(self.board[row_idx][3].as_ref()),
        ];

        matching_pieces(&row)
    }

    fn winning_col(&self, square: &Coord) -> bool {
        let col_idx = square.1;
        let column = [
            check_for_piece!(self.board[0][col_idx].as_ref()),
            check_for_piece!(self.board[1][col_idx].as_ref()),
            check_for_piece!(self.board[2][col_idx].as_ref()),
            check_for_piece!(self.board[3][col_idx].as_ref()),
        ];

        matching_pieces(&column)
    }

    // Check for win along the top left to bottom right diagonal
    fn winning_downward_diagonal(&self, square: &Coord) -> bool {
        // Bail out if played position is not on diagonal
        if square.0 != square.1 {
            return false;
        }

        let column = [
            check_for_piece!(self.board[0][0].as_ref()),
            check_for_piece!(self.board[1][1].as_ref()),
            check_for_piece!(self.board[2][2].as_ref()),
            check_for_piece!(self.board[3][3].as_ref()),
        ];

        matching_pieces(&column)
    }

    // Check for win along the bottom left to top right diagonal
    fn winning_upward_diagonal(&self, square: &Coord) -> bool {
        // Bail out if played position is not on diagonal
        if (square.0 + square.1) != 3 {
            return false;
        }
        let column = [
            check_for_piece!(self.board[0][3].as_ref()),
            check_for_piece!(self.board[1][2].as_ref()),
            check_for_piece!(self.board[2][1].as_ref()),
            check_for_piece!(self.board[3][0].as_ref()),
        ];

        matching_pieces(&column)
    }

    // TODO: do placement and staging computation in single pass
    fn monte_stage_scores(&self) -> Vec<i32> {
        let mut scores: Vec<i32> = [0].repeat(self.pieces.len());

        let perf = web_sys::window().unwrap().performance().unwrap();
        let start = perf.now();

        for n in 0..SIMULATIONS {
            if (perf.now() - start) > MAX_RUNTIME_MS {
                seed::log!("Bailing staging", n);
                break;
            }
            let mut game = self.clone();
            let idx = game.rng.gen_range(0, game.pieces.len());
            let piece = game.pieces.remove(idx);
            // println!("Randomly choosing piece {}", piece.display());
            scores[idx] += game.placement_score(piece, Player::Human);
        }

        scores
    }

    fn monte_placement_scores(&self, piece: &Piece) -> HashMap<Coord, i32> {
        let mut scores: HashMap<Coord, i32> = HashMap::new();
        let available = self.empty_squares();

        let perf = web_sys::window().unwrap().performance().unwrap();
        let start = perf.now();

        for n in 0..SIMULATIONS {
            if (perf.now() - start) > MAX_RUNTIME_MS {
                seed::log!("Bailing placement", n);
                break;
            }
            let mut game = self.clone();
            let square = available.choose(&mut game.rng).unwrap();
            *scores.entry(*square).or_default() +=
                game.placement_score(piece.clone(), Player::Machine);
        }

        scores
    }

    fn placement_score(&mut self, piece: Piece, player: Player) -> i32 {
        let mut piece = Some(piece);
        for square in self.empty_squares() {
            self.board[square.0][square.1] = piece;
            if self.is_win(&square) {
                // println!("Found win");
                // print_board(&self.board);

                return match player {
                    Player::Machine => 1,
                    Player::Human => -1,
                };
            }
            piece = self.board[square.0][square.1].take();
        }

        let available = self.empty_squares();
        let square = available.choose(&mut self.rng).unwrap();
        self.board[square.0][square.1] = piece;
        // print_board(&self.board);
        // println!("{:?} played in square {:?}", player, square);

        self.stage_score(player)
    }

    fn stage_score(&mut self, player: Player) -> i32 {
        if self.pieces.is_empty() {
            return 0;
        }

        let idx = self.rng.gen_range(0, self.pieces.len());
        let piece = self.pieces.remove(idx);
        self.placement_score(piece, player.toggle())
    }
}

const SIMULATIONS: u32 = 10000;
const MAX_RUNTIME_MS: f64 = 10_000_f64;

#[derive(Debug, Clone)]
pub(crate) enum Play {
    /// Player selects piece for other player to place
    Staged(Player, Piece),
    /// Player placed piece upon board
    Placed(Player),
    Transitioning,
    Finished(Resolution),
}

#[derive(Debug, Clone)]
pub(crate) enum Resolution {
    Win(Player),
    Draw,
}

fn print_board(board: &Board) {
    let lines: Vec<String> = board
        .iter()
        .map(|row| {
            let cells: Vec<String> = row
                .iter()
                .map(|piece| {
                    if let Some(piece) = piece {
                        piece.display()
                    } else {
                        "  ".to_string()
                    }
                })
                .collect();
            format!("│ {} │", cells.join(" │ "))
        })
        .collect();
    println!("┌――――┬――――┬――――┬――――┐");
    println!("{}", lines.join("\n├――――┼――――┼――――┼――――┤\n"));
    println!("└――――┴――――┴――――┴――――┘");
}

// const MAX_DEPTH: usize = 8;

// fn play(board: Board, pieces: &Vec<Piece>, current_player: &Player, level: usize) -> i64 {
//     if pieces.len() == 0 || level >= MAX_DEPTH {
//         return 0;
//     }

//     // Skip win check if there aren't enough pieces for it to be possible
//     if pieces.len() < 10 && is_win(&board) {
//         match current_player {
//             Player::Machine => return 1,
//             Player::Human => return -1,
//         }
//     };

//     let mut rng = rand::thread_rng();
//     let range = 0..board.len();
//     let mut moves: Vec<(usize, usize, &Piece)> = range
//         .clone()
//         .flat_map(|row_idx| range.clone().map(move |square_idx| (row_idx, square_idx)))
//         .filter(|(row_idx, square_idx)| !board[*row_idx][*square_idx].is_some())
//         .flat_map(|(row_idx, square_idx)| {
//             pieces.iter().map(move |piece| (row_idx, square_idx, piece))
//         })
//         .collect();
//     (&mut moves).shuffle(&mut rng);

//     moves
//         .into_par_iter()
//         .take(20)
//         .map(|(row_idx, square_idx, piece)| {
//             let remaining = pieces.clone().into_iter().filter(|p| p != piece).collect();
//             let mut board = board.clone();
//             board[row_idx][square_idx].replace(piece.clone());
//             let path_score = play(board, &remaining, &current_player.toggle(), level + 1);

//             if level < 3 {
//                 dbg!(level, path_score, row_idx, square_idx);
//             }

//             path_score
//         })
//         .sum()
// }

fn empty_board() -> Board {
    let row = [None, None, None, None];

    [row.clone(), row.clone(), row.clone(), row.clone()]
}

const BOARD_SIZE: usize = 4;
type Board = [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE];
pub(crate) type Coord = (usize, usize);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Piece {
    pub(crate) color: Color,
    pub(crate) height: Height,
    pub(crate) density: Density,
    pub(crate) shape: Shape,
}

impl Piece {
    pub(crate) fn display(&self) -> String {
        let first = match (&self.color, &self.height) {
            (Dark, Tall) => "D",
            (Dark, Short) => "d",
            (Light, Tall) => "L",
            (Light, Short) => "l",
        };
        let second = match (&self.density, &self.shape) {
            (Hollow, Round) => "○",
            (Hollow, Square) => "□",
            (Solid, Round) => "●",
            (Solid, Square) => "■",
        };

        format!("{}{}", first, second)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Height {
    Tall,
    Short,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Color {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Density {
    Solid,
    Hollow,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Shape {
    Round,
    Square,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Player {
    Machine,
    Human,
}

impl Player {
    fn toggle(&self) -> Self {
        match self {
            Player::Machine => Player::Human,
            Player::Human => Player::Machine,
        }
    }
}

fn matching_pieces(pieces: &[&Piece; 4]) -> bool {
    let first_piece = pieces[0];
    let mut color = Some(&first_piece.color);
    let mut shape = Some(&first_piece.shape);
    let mut density = Some(&first_piece.density);
    let mut height = Some(&first_piece.height);
    for piece in pieces.iter().skip(1) {
        if color != Some(&piece.color) {
            color.take();
        }
        if height != Some(&piece.height) {
            height.take();
        }
        if shape != Some(&piece.shape) {
            shape.take();
        }
        if density != Some(&piece.density) {
            density.take();
        }

        if color.is_none() && shape.is_none() && density.is_none() && height.is_none() {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_win(board: Board, square: &Coord) -> bool {
        let game = Game {
            board,
            pieces: vec![],
            last_play: Play::Transitioning,
            rng: rand::thread_rng(),
        };

        game.is_win(square)
    }

    #[test]
    fn tall_win_test() {
        let mut board = empty_board();
        board[0][0] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Solid,
            shape: Round,
        });
        board[0][1] = Some(Piece {
            height: Tall,
            color: Light,
            density: Solid,
            shape: Round,
        });
        board[0][2] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Hollow,
            shape: Round,
        });
        board[0][3] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Solid,
            shape: Square,
        });

        assert_eq!(is_win(board, &(0, 0)), true);
    }

    #[test]
    fn dark_win_test() {
        let mut board = empty_board();
        board[0][0] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Solid,
            shape: Round,
        });
        board[1][0] = Some(Piece {
            height: Short,
            color: Dark,
            density: Solid,
            shape: Round,
        });
        board[2][0] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Hollow,
            shape: Round,
        });
        board[3][0] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Solid,
            shape: Square,
        });

        assert_eq!(is_win(board, &(0, 0)), true);
    }

    #[test]
    fn diagonal_win_test() {
        let mut board = empty_board();
        board[0][0] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Solid,
            shape: Round,
        });
        board[1][1] = Some(Piece {
            height: Short,
            color: Dark,
            density: Solid,
            shape: Round,
        });
        board[2][2] = Some(Piece {
            height: Tall,
            color: Dark,
            density: Hollow,
            shape: Round,
        });
        board[3][3] = Some(Piece {
            height: Tall,
            color: Light,
            density: Solid,
            shape: Round,
        });

        assert_eq!(is_win(board.clone(), &(0, 0)), true);
        assert_eq!(is_win(board, &(0, 3)), false);
    }

    #[test]
    fn empty_is_not_win_test() {
        let board = empty_board();

        assert_eq!(is_win(board, &(0, 0)), false);
    }
}