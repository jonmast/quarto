use seed::prelude::*;
use seed::{div, C};

mod game;

use game::Piece;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model::default()
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
type Model = game::Game;

// ------ ------
//    Update
// ------ ------

#[derive(Copy, Clone)]
enum Msg {
    Stage(usize),
    Place(game::Coord),
    Advance,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Stage(idx) => {
            model.try_stage_piece(idx);

            orders.send_msg(Msg::Advance);
        }
        Msg::Place(square) => {
            seed::log!("Placing the piece");
            model.try_place_piece(square);
            advance_state(model);
        }
        Msg::Advance => {
            if advance_state(model) {
                orders.send_msg(Msg::Advance);
            }
        }
    }
}

fn advance_state(model: &mut Model) -> bool {
    seed::log!("Advancing...");
    if model.is_over() {
        seed::log!("Game over");
        return false;
    }

    match model.last_play {
        game::Play::Placed(game::Player::Human) | game::Play::Staged(game::Player::Machine, _) => {
            seed::log!("Going to user input");
            return false;
        }
        _ => {
            model.tick();
            return true;
        }
    }
}

// ------ ------
//     View
// ------ ------
fn view(model: &Model) -> Node<Msg> {
    seed::log!("re-rendering");
    div![
        div![
            C!["board"],
            model.board.iter().enumerate().map(|(row_idx, row)| {
                div![
                    C!["row"],
                    row.iter().enumerate().map(|(col_idx, cell)| {
                        div![
                            C!["cell"],
                            ev(Ev::Click, move |_| Msg::Place((row_idx, col_idx))),
                            cell.as_ref().map(display_piece)
                        ]
                    })
                ]
            })
        ],
        div![display_state(&model.last_play)],
        div![
            C!["unplayed"],
            model.pieces.iter().enumerate().map(|(idx, piece)| {
                div![
                    ev(Ev::Click, move |_| Msg::Stage(idx)),
                    C!["available-slot"],
                    display_piece(piece)
                ]
            })
        ]
    ]
}

fn display_piece(piece: &Piece) -> Node<Msg> {
    div![
        C!["piece", piece_classes(piece)],
        div![C!["face face-top"]],
        div![C!["face face-front"]],
        div![C!["face face-right"]],
    ]
}

fn display_state(play: &game::Play) -> Node<Msg> {
    use game::Play::*;
    use game::Player;

    match play {
        Finished(game::Resolution::Draw) => div!["Cat got it!"],
        Finished(game::Resolution::Win(player)) => match player {
            Player::Human => div!["You win!"],
            Player::Machine => div!["You lose!"],
        },
        Placed(Player::Human) => div!["Please select a piece for the computer to play"],
        Staged(Player::Machine, piece) => div![
            "Computer selected:",
            div![C!["piece-staging"], display_piece(piece)],
            "Please select square to play piece",
        ],
        _ => div!["Thinking..."],
    }
}

fn piece_classes(piece: &Piece) -> Vec<&'static str> {
    use game::Color::*;
    use game::Density::*;
    use game::Height::*;
    use game::Shape::*;

    vec![
        match piece.density {
            Hollow => "hollow",
            Solid => "solid",
        },
        match piece.color {
            Dark => "dark",
            Light => "light",
        },
        match piece.height {
            Tall => "tall",
            Short => "short",
        },
        match piece.shape {
            Round => "round",
            Square => "square",
        },
    ]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
