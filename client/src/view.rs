
use coffee::{Game, Timer};
use coffee::graphics::*;
use coffee::input::{Input, keyboard, mouse};
use coffee::load::Task;
use coffee::ui::{button, Button, Element, Renderer, Row, UserInterface, Text};

use std::collections::HashMap;

use super::appstate::{AppState, StateSet};
use super::floatingtetromino::FloatingTetromino;
use super::ltpcontroller::LtpController;
use super::states::*;

use lits;
use lits::{Colour, Player, Tetromino};
use utils::notate::Notate;

///
/// An encapsulation of a full game state and interface state for The Battle of LITS.
///
/// Game objects:
/// - a game, which is a view on a LITS game with linear history; and 
/// - a floating tetromino, which can be picked up from the piece pool and dropped onto the board.
///
/// Engine objects:
/// - a handle, which is the UUID pointing to a response expected from the engine; and 
/// - an app state, which is an additive set of application states describing modifications to
///     functionality.
///
pub struct View 
{
    // Game objects.

    game: lits::Game,
    backup_copy: lits::Game,
    pub floating_tetromino: Option<FloatingTetromino>,

    // Engine handles.

    controller: LtpController,
    app_state: StateSet,

    // UI objects.
    
    input_state: InputState,
    window_size: WindowSize,

    cancel_search_button: button::State,
    gen_move_button: button::State,
    undo_move_button: button::State,
    new_game_button: button::State,
    setup_mode_button: button::State,
    cancel_setup_button: button::State,
    confirm_setup_button: button::State
}

impl std::ops::Drop for View 
{
    fn drop (self: & mut View) 
    {
        self.controller.halt();
    }
}

impl View 
{
    ///
    /// Blocks on wait-play by first sending an engine cancellation.
    ///
    pub fn cancel_and_play (& mut self)
    {
        self.controller.cmd_cancel();
        while ! self.wait_to_play() 
        {
            continue;
        }
    }

    ///
    /// Cleans up the resources used by piece mode and exits it.
    ///
    pub fn clean_up_piece_mode (& mut self)
    {
        self.floating_tetromino = None;
        self.app_state.remove(& AppState::PieceMode);
    }

    ///
    /// Initiates piece mode.
    ///
    pub fn enter_piece_mode_with (& mut self, colour: & Colour)
    {
        let rel_x = (self.input_state.cursor_position.x - self.window_size.get_board_corner().x) / self.window_size.get_tile_size();
        let rel_y = (self.input_state.cursor_position.y - self.window_size.get_board_corner().y) / self.window_size.get_tile_size();

        self.floating_tetromino = Some(
            FloatingTetromino::new(
                & Tetromino::get_reference_tetromino(& colour, & lits::Point::new(rel_x.round() as i32, rel_y.round() as i32)),
                rel_x,
                rel_y 
            )
        );

        self.app_state.insert(AppState::PieceMode);
    }

    ///
    /// Requests the engine to generate a move, and swaps to Waiting.
    ///
    pub fn gen_move (& mut self)
    {
        self.controller.cmd_gen_move(& self.game.to_move());

        self.clean_up_piece_mode();
        self.app_state.insert(AppState::Waiting);
    }

    ///
    /// Starts a new game.
    ///
    pub fn new_game (& mut self) 
    {
        let _ = self.controller.cmd_new_game();
        self.game = lits::Game::new();

        self.clean_up_piece_mode();
        self.app_state.clear();
    }

    ///
    /// Cancels the setup, returning to the previous position.
    ///
    pub fn setup_cancel (& mut self)
    {
        self.game = self.backup_copy.clone();

        self.app_state.remove(& AppState::PieceMode);
        self.app_state.remove(& AppState::BoardSetupMode);
    }
        
    ///
    /// Pushes the setup to the engine and enters InGame.
    ///
    pub fn setup_confirm (& mut self)
    {
        self.game = lits::Game::parse(& self.game.get_board().notate()).unwrap();
        let _ = self.controller.cmd_apply_setup(self.game.get_board_base());

        self.app_state.remove(& AppState::PieceMode);
        self.app_state.remove(& AppState::BoardSetupMode);
    }

    ///
    /// Saves the game into the backup copy slot and enters setup mode.
    ///
    pub fn swap_to_setup (& mut self) 
    {
        self.backup_copy = self.game.clone();
        self.game = lits::Game::new();

        self.app_state.remove(& AppState::PieceMode);
        self.app_state.insert(AppState::BoardSetupMode);
    }

    ///
    /// Determines the point the mouse is over, if any.
    ///
    pub fn tile_at_mouse (& mut self) -> Option<lits::Point>
    {
        let corner = self.window_size.get_board_corner();
        let side = self.window_size.get_tile_size();

        let mouse_point = Point::new(self.input_state.cursor_position.x, self.input_state.cursor_position.y);

        // Compute the float game coord, which is the fuzzy tile index.

        let rel_x = ((mouse_point.x - corner.x) / side).floor() as i32;
        let rel_y = ((mouse_point.y - corner.y) / side).floor() as i32;

        if 0 <= rel_x && rel_x < 10 && 0 <= rel_y && rel_y < 10 
        {
            return Some(lits::Point::new(rel_x, rel_y));
        }
        None
    }

    ///
    /// Tries to place the floating piece; if it works, goes to InGame 
    /// and stays in PieceMode otherwise.
    ///
    pub fn try_placing_piece (& mut self)
    {
        let floater = self.floating_tetromino.as_mut().unwrap();
        if self.game.apply(& floater.tetromino()).is_ok()
        {
            self.controller.cmd_play(& floater.tetromino());
            self.clean_up_piece_mode();
        }
    }

    ///
    /// Tries to undo the last move.
    ///
    pub fn try_undo (& mut self) 
    {
        if ! self.game.get_history().is_empty()
        {
            if self.game.undo().is_ok()
            {
                self.controller.cmd_undo();
                self.app_state.clear();
            }
        }
    }

    ///
    /// Using the window state, calculate the new floating 
    /// relative board coordinate for the floating piece. If 
    /// the piece is snapping, update the anchor on it.
    ///
    pub fn update_floater_position (& mut self)
    {
        if self.floating_tetromino.is_some()
        {
            let floater = self.floating_tetromino.as_mut().unwrap();
            
            let corner = self.window_size.get_board_corner();
            let side = self.window_size.get_tile_size();

            let mouse_point = Point::new(self.input_state.cursor_position.x, self.input_state.cursor_position.y);

            // Compute the float game coord, which is the fuzzy tile index.

            let rel_x = (mouse_point.x - corner.x) / side;
            let rel_y = (mouse_point.y - corner.y) / side;

            * floater.x() = rel_x;
            * floater.y() = rel_y;

            // If the piece snapped, then move its anchor.

            if let Some(anchor) = floater.snap()
            {
                floater.set_anchor(anchor);
            }
        }
    }

    ///
    /// The transition function from Waiting to InGame;
    /// when it receives an engine response, it plays it 
    /// into the position and moves to InGame.
    ///
    pub fn wait_to_play (& mut self) -> bool
    {
        // Wait for a response.

        let engine_response = self.controller.poll_response();
        let response = match engine_response 
        {
            Ok(string) => Some(string),
            Err(_)     => None
        };

        if response.is_some()
        {
            // Parse the response to get the tetromino.

            let tetromino = Tetromino::parse(& response.unwrap()).unwrap();

            // Play the move and update the app state.

            if self.game.apply(& tetromino).is_ok()
            {
                self.controller.cmd_play(& tetromino);
                self.app_state.remove(& AppState::Waiting);
            }

            return true;
        }
        false
    }
}

impl Game for View 
{
    type Input = InputState;
    type LoadingScreen = ();

    fn draw (& mut self, frame: & mut Frame, timer: & Timer)
    {
        if ! timer.has_ticked()
        {
            return;
        }

        let fg      = Color::from_rgb_u32(0x303034);
        let bg      = Color::from_rgb_u32(0x202028);
        let border  = Color::from_rgb_u32(0x747070);
        let colours = HashMap::from([
            (Colour::L, Color::from_rgb_u32(0xDC2430)),
            (Colour::I, Color::from_rgb_u32(0xEDC830)),
            (Colour::T, Color::from_rgb_u32(0x20B810)),
            (Colour::S, Color::from_rgb_u32(0x18B8D8)),
            (Colour::None, Color::from_rgb_u32(0xCCCCCC))
        ]);

        frame.clear(bg);
        
        // Draw the board; first draw the base, then draw 
        // the Xs and Os, then draw non-null colours.

        let board = self.game.get_board();

        let corner = self.window_size.get_board_corner();
        let side = self.window_size.get_tile_size();
        let boardside = self.window_size.get_board_size();
        let borderwidth = self.window_size.get_border_width();

        let mut mesh = Mesh::new();

        // Base border.
        
        mesh.fill(
            Shape::Rectangle(
                Rectangle 
                {
                    x: corner.x - borderwidth,
                    y: corner.y - borderwidth,
                    width: boardside + 2.0 * borderwidth,
                    height: boardside + 2.0 * borderwidth
                }
            ),
            border 
        );

        // Background tile, then player tiles, then colour tiles over them.

        for i in 0 .. 10
        {
            for j in 0 .. 10
            {
                let colour = board.colour_at(i, j);
                
                mesh.fill(
                    Shape::Rectangle(
                        Rectangle 
                        {
                            x: corner.x + (i as f32) * side + (borderwidth / 2.0),
                            y: corner.y + (j as f32) * side + (borderwidth / 2.0),
                            width: side - (borderwidth / 2.0),
                            height: side - (borderwidth / 2.0)
                        }
                    ),
                    * colours.get(& Colour::None).unwrap()
                );


                if colour != Colour::None 
                {
                    mesh.fill(
                        Shape::Rectangle(
                            Rectangle 
                            {
                                x: corner.x + (i as f32) * side + (borderwidth / 2.0),
                                y: corner.y + (j as f32) * side + (borderwidth / 2.0),
                                width: side - (borderwidth / 2.0),
                                height: side - (borderwidth / 2.0)
                            }
                        ),
                        * colours.get(& colour).unwrap()
                    );
                }

                let player = board.player_at(i, j);
                
                if player == Player::X 
                {
                    // Why am I like this?

                    mesh.fill(
                        Shape::Polyline
                        {
                            points: 
                                vec!
                                [
                                    Point::new(0.1, 0.2),
                                    Point::new(0.2, 0.1),
                                    Point::new(0.5, 0.4), 
                                    Point::new(0.8, 0.1),
                                    Point::new(0.9, 0.2),
                                    Point::new(0.6, 0.5),
                                    Point::new(0.9, 0.8),
                                    Point::new(0.8, 0.9),
                                    Point::new(0.5, 0.6),
                                    Point::new(0.2, 0.9),
                                    Point::new(0.1, 0.8),
                                    Point::new(0.4, 0.5),
                                    Point::new(0.1, 0.2)
                                ]
                                .iter()
                                .map(|p| Point::new(corner.x + (borderwidth / 2.0) + p.x * (side - borderwidth), corner.y + (borderwidth / 2.0) + p.y * (side - borderwidth)))
                                .map(|p| Point::new(p.x + (i as f32) * side, p.y + (j as f32) * side) )
                                .collect::<Vec<Point>>()
                        },
                        fg 
                    );
                }
                else if player == Player::O 
                {
                    mesh.stroke(
                        Shape::Circle 
                        {
                            radius: (side - 5.0 * borderwidth) / 2.0,
                            center: Point::new(corner.x + (i as f32 + 0.5) * side, corner.y + (j as f32 + 0.5) * side)
                        },
                        fg,
                        2.0 * borderwidth
                    );
                }
            }
        }

        // Now handle the potential floating piece.
        // The piece is drawn; then if it has a snapping 
        // position underneath it that is also a valid place 
        // to put the piece, then highlight those squares 
        // on the gameboard.
        
        if self.floating_tetromino.is_some()
        {
            let floater = self.floating_tetromino.as_mut().unwrap();

            let true_x = corner.x + floater.x().to_owned() * side;
            let true_y = corner.y + floater.y().to_owned() * side;

            let alpha = 0.6;
            let colour_ref = floater.tetromino().colour();
            let colour_old = colours.get(& colour_ref).unwrap();
            let colour_new = Color::new(colour_old.r, colour_old.g, colour_old.b, alpha);

            // If the tetromino could be played where it's currently snapped to, brighten
            // the squares that correspond to its snap position.

            if self.game.get_board().validate_tetromino(& floater.tetromino()).is_ok()
            {
                let glow = Color::new(0.0, 0.0, 0.0, 0.2);
                
                for point in & floater.tetromino().points_real()
                {
                    let i = point.x();
                    let j = point.y();

                    mesh.fill(
                        Shape::Rectangle(
                            Rectangle 
                            {
                                x: corner.x + (i as f32) * side,
                                y: corner.y + (j as f32) * side,
                                width: side,
                                height: side 
                            }
                        ),
                        glow 
                    );
                }
            }

            // Display the true floating position of the tetromino under the mouse, with 
            // some degree of transparency.

            for point in floater.tetromino().points()
            {
                let i = point.x();
                let j = point.y();

                mesh.fill(
                    Shape::Rectangle(
                        Rectangle
                        {
                            x: true_x + (i as f32) * side + (borderwidth / 2.0),
                            y: true_y + (j as f32) * side + (borderwidth / 2.0),
                            width: side - (borderwidth / 2.0),
                            height: side - (borderwidth / 2.0)
                        }
                    ),
                    colour_new 
                );
            }
        }

        mesh.draw(& mut frame.as_target());
    }

    fn interact (& mut self, input: & mut InputState, window: & mut Window)
    {
        // Update values.

        self.input_state = input.clone();
        self.window_size = WindowSize::new(window.width(), window.height());

        if self.app_state.contains(& AppState::Waiting)
        {
            // The only thing you can do in the waiting state is cancel an engine operation.
            // Otherwise, it polls to see if its desired response has appeared yet.

            if self.input_state.keys_pressed.contains(& keyboard::KeyCode::C)
                && self.input_state.keys_pressed.contains(& keyboard::KeyCode::LControl)
            {
                self.cancel_and_play();
            }
            else 
            {
                self.wait_to_play();
            }
        }
        else if self.app_state.contains(& AppState::PieceMode)
        { 
            // Set the relative board float coordinate for the binded piece, using the 
            // calculated bounds from the window size to compute the position.

            self.update_floater_position();

            // On pressing enter, cycle to the next transformation of this piece.

            if self.input_state.keys_pressed.contains(& keyboard::KeyCode::Return)
            {
                self.floating_tetromino.as_mut().unwrap().next();
            }
            else if self.input_state.mouse_scroll_wheel.y > 0.0
            {
                let y = self.input_state.mouse_scroll_wheel.y.round() as i32;
                for _ in 0 .. y 
                {
                    self.floating_tetromino.as_mut().unwrap().next();
                }
            }
            else if self.input_state.mouse_scroll_wheel.y < 0.0 
            {
                let y = self.input_state.mouse_scroll_wheel.y.round() as i32;
                for _ in 0 .. -y 
                {
                    self.floating_tetromino.as_mut().unwrap().prev();
                }
            }

            // Otherwise, handle exit conditions provided by the mouse.
            
            if self.input_state.mouse_buttons_pressed.contains(& mouse::Button::Right)
            {
                self.input_state.mouse_buttons_pressed.remove(& mouse::Button::Right);
                self.clean_up_piece_mode();
            }
            else if self.input_state.mouse_buttons_pressed.contains(& mouse::Button::Left)
            {
                self.input_state.mouse_buttons_pressed.remove(& mouse::Button::Left);
                self.try_placing_piece();
            }
        }
        else if self.app_state.contains(& AppState::BoardSetupMode)
        {
            // Left-clicking a tile cycles its colour, right-clicking a tile cycles 
            // its player.
            
            let point = self.tile_at_mouse();
            if point.is_some()
            {
                let point = point.unwrap();
                if self.input_state.mouse_scroll_wheel.y > 0.0
                {
                    let y = self.input_state.mouse_scroll_wheel.y.round() as i32;
                    for _ in 0 .. y 
                    {
                        self.game.cycle_colour(point.x(), point.y());
                    }
                }
                else if self.input_state.mouse_scroll_wheel.y < 0.0 
                {
                    let y = self.input_state.mouse_scroll_wheel.y.round() as i32;
                    for _ in 0 .. -y 
                    {
                        self.game.cycle_player(point.x(), point.y());
                    }
                }
            }
        }
        else
        {
            if self.input_state.keys_pressed.contains(& keyboard::KeyCode::M)
                && self.input_state.keys_pressed.contains(& keyboard::KeyCode::LControl)
            {
                self.gen_move();
            }
            
            let colour_to_keycode = HashMap::from([
                (Colour::L, keyboard::KeyCode::L),
                (Colour::I, keyboard::KeyCode::I),
                (Colour::T, keyboard::KeyCode::T),
                (Colour::S, keyboard::KeyCode::S)
            ]);

            for colour in [Colour::L, Colour::I, Colour::T, Colour::S] 
            {
                if self.input_state.keys_pressed.contains(& colour_to_keycode.get(& colour).unwrap())
                    && self.game.get_board().remaining_of(& colour) > 0 
                {
                    self.enter_piece_mode_with(& colour);
                }
            }
        }
    }

    fn load (_window: & Window) -> Task<View>
    {
        Task::succeed(
            || View 
            {
                game: lits::Game::new(),
                backup_copy: lits::Game::new(),
                floating_tetromino: None,
                controller: LtpController::new(),
                app_state: StateSet::new(),
                input_state: InputState::new(),
                window_size: WindowSize::new(0.0, 0.0),
                cancel_search_button: button::State::new(),
                gen_move_button: button::State::new(),
                undo_move_button: button::State::new(),
                new_game_button: button::State::new(),
                setup_mode_button: button::State::new(),
                cancel_setup_button: button::State::new(),
                confirm_setup_button: button::State::new()
            }
        )
    }
}

impl UserInterface for View 
{
    type Renderer = Renderer;
    type Message = EventState;

    fn layout (& mut self, _window: & Window) -> Element<EventState>
    {
        let bw = self.window_size.get_button_height().round() as u32;

        if self.app_state.contains(& AppState::Waiting)
        {
            return Row::new().padding(self.window_size.get_border_width().round() as u32)
                .max_height(bw)
                .push(
                    Button::new(& mut self.cancel_search_button, "Cancel Search")
                        .on_press(EventState::CancelSearchButton).width(bw)
                )
                .into();
        }
        else if self.app_state.contains(& AppState::BoardSetupMode)
        {
            let pt_text = match self.tile_at_mouse()
            {
                Some(point) => point.to_string(),
                None        => "none".to_owned()
            };

            return Row::new().padding(self.window_size.get_border_width().round() as u32)
                .max_height(bw)
                .push(
                    Button::new(& mut self.cancel_setup_button, "Discard Setup")
                        .on_press(EventState::CancelSetupButton).width(bw)
                )
                .push(
                    Button::new(& mut self.confirm_setup_button, "Confirm Setup")
                        .on_press(EventState::ConfirmSetupButton).width(bw)
                )
                .push(
                    Text::new(& pt_text.clone())
                )
                .into();
        }
        else 
        {
            return Row::new().padding(self.window_size.get_border_width().round() as u32)
                .max_height(bw)
                .push(
                    Button::new(& mut self.gen_move_button, "Generate Move")
                        .on_press(EventState::PlayMoveButton).width(bw)
                )
                .push(
                    Button::new(& mut self.undo_move_button, "Undo Move")
                        .on_press(EventState::UndoMoveButton).width(bw)
                )
                .push(
                    Button::new(& mut self.new_game_button, "New Game")
                        .on_press(EventState::NewGameButton).width(bw)
                )
                .push(
                    Button::new(& mut self.setup_mode_button, "Enter Setup Mode")
                        .on_press(EventState::SetupModeButton).width(bw)
                )
                .into();
        }
    }

    fn react (& mut self, message: EventState, _window: & mut Window)
    {
        match message 
        {
            EventState::NewGameButton      => self.new_game(),
            EventState::SetupModeButton    => self.swap_to_setup(),
            EventState::PlayMoveButton     => self.gen_move(),
            EventState::CancelSearchButton => self.cancel_and_play(),
            EventState::ConfirmSetupButton => self.setup_confirm(),
            EventState::CancelSetupButton  => self.setup_cancel(),
            EventState::UndoMoveButton     => self.try_undo()
        };
    }
}
