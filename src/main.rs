use rand::seq::SliceRandom;
use rayon::prelude::*;

use Color::*;
use Density::*;
use Height::*;
use Shape::*;

fn main() {
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

    let current_player = Player::A;
    play(empty_board(), &pieces, &current_player, 1);
}

const MAX_DEPTH: usize = 8;

fn play(board: Board, pieces: &Vec<Piece>, current_player: &Player, level: usize) -> i64 {
    if pieces.len() == 0 || level >= MAX_DEPTH {
        return 0;
    }

    // Skip win check if there aren't enough pieces for it to be possible
    if pieces.len() < 10 && is_win(&board) {
        match current_player {
            Player::A => return 1,
            Player::B => return -1,
        }
    };

    let mut rng = rand::thread_rng();
    let range = 0..board.len();
    let mut moves: Vec<(usize, usize, &Piece)> = range
        .clone()
        .flat_map(|row_idx| range.clone().map(move |square_idx| (row_idx, square_idx)))
        .filter(|(row_idx, square_idx)| !board[*row_idx][*square_idx].is_some())
        .flat_map(|(row_idx, square_idx)| {
            pieces.iter().map(move |piece| (row_idx, square_idx, piece))
        })
        .collect();
    (&mut moves).shuffle(&mut rng);

    moves
        .into_par_iter()
        .take(20)
        .map(|(row_idx, square_idx, piece)| {
            let remaining = pieces.clone().into_iter().filter(|p| p != piece).collect();
            let mut board = board.clone();
            board[row_idx][square_idx].replace(piece.clone());
            let path_score = play(board, &remaining, &current_player.toggle(), level + 1);

            if level < 3 {
                dbg!(level, path_score, row_idx, square_idx);
            }

            path_score
        })
        .sum()
}

fn empty_board() -> Board {
    let row = [None, None, None, None];

    [row.clone(), row.clone(), row.clone(), row.clone()]
}

type Board = [[Option<Piece>; 4]; 4];

#[derive(Clone, PartialEq)]
struct Piece {
    height: Height,
    color: Color,
    density: Density,
    shape: Shape,
}

#[derive(Clone, PartialEq)]
enum Height {
    Tall,
    Short,
}

#[derive(Clone, PartialEq)]
enum Color {
    Dark,
    Light,
}

#[derive(Clone, PartialEq)]
enum Density {
    Solid,
    Hollow,
}

#[derive(Clone, PartialEq)]
enum Shape {
    Round,
    Square,
}

enum Player {
    A,
    B,
}

impl Player {
    fn toggle(&self) -> Self {
        match self {
            Player::A => Player::B,
            Player::B => Player::A,
        }
    }
}

fn is_win(board: &Board) -> bool {
    for row in 0..board.len() {
        if winning_row(board, row) {
            return true;
        }
    }

    for col in 0..board.len() {
        if winning_col(board, col) {
            return true;
        }
    }

    if left_right_diaganal_win(board) || right_left_diaganal_win(board) {
        return true;
    }

    false
}

fn winning_row(board: &Board, row: usize) -> bool {
    let column = [
        board[row][0].as_ref(),
        board[row][1].as_ref(),
        board[row][2].as_ref(),
        board[row][3].as_ref(),
    ];

    matching_pieces(&column)
}

fn winning_col(board: &Board, col: usize) -> bool {
    let column = [
        board[0][col].as_ref(),
        board[1][col].as_ref(),
        board[2][col].as_ref(),
        board[3][col].as_ref(),
    ];

    matching_pieces(&column)
}

fn left_right_diaganal_win(board: &Board) -> bool {
    let column = [
        board[0][0].as_ref(),
        board[1][1].as_ref(),
        board[2][2].as_ref(),
        board[3][3].as_ref(),
    ];

    matching_pieces(&column)
}

fn right_left_diaganal_win(board: &Board) -> bool {
    let column = [
        board[0][3].as_ref(),
        board[1][2].as_ref(),
        board[2][1].as_ref(),
        board[3][0].as_ref(),
    ];

    matching_pieces(&column)
}

fn matching_pieces(pieces: &[Option<&Piece>; 4]) -> bool {
    match &pieces[0] {
        Some(first_piece) => {
            let mut color = Some(&first_piece.color);
            let mut shape = Some(&first_piece.shape);
            let mut density = Some(&first_piece.density);
            let mut height = Some(&first_piece.height);
            for piece in pieces.iter().skip(1) {
                match piece {
                    Some(piece) => {
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
                    }
                    None => return false,
                }

                if color.is_none() && shape.is_none() && density.is_none() && height.is_none() {
                    return false;
                }
            }
            return true;
        }
        None => return false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(is_win(&board), true);
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

        assert_eq!(is_win(&board), true);
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

        assert_eq!(is_win(&board), true);
    }

    #[test]
    fn empty_is_not_win_test() {
        let board = empty_board();

        assert_eq!(is_win(&board), false);
    }
}
