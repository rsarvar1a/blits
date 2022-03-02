
use coffee::input::{ButtonState, Event, Input, keyboard, mouse};

use std::collections::HashSet;

///
/// An encapsulation of input mechanisms used by this game.
///
#[derive(Clone, Debug)]
pub struct InputState
{
    pub cursor_position: coffee::graphics::Point,
    pub keys_pressed: HashSet<keyboard::KeyCode>,
    pub mouse_buttons_pressed: HashSet<mouse::Button>,
    pub mouse_scroll_wheel: coffee::graphics::Point
}

impl Input for InputState 
{
    fn clear (& mut self)
    {
        self.mouse_scroll_wheel = coffee::graphics::Point::new(0.0, 0.0);
    }

    fn new () -> InputState 
    {
        InputState 
        {
            cursor_position: coffee::graphics::Point::new(0.0, 0.0),
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_scroll_wheel: coffee::graphics::Point::new(0.0, 0.0),
        }
    }

    fn update (& mut self, event: Event)
    {
        match event 
        {
            Event::Mouse(mouse_event) => match mouse_event 
            {
                mouse::Event::CursorMoved { x, y } => 
                {
                    self.cursor_position = coffee::graphics::Point::new(x, y);
                },
                mouse::Event::Input { state, button } => match state 
                {
                    ButtonState::Pressed => 
                    {
                        self.mouse_buttons_pressed.insert(button);
                    },
                    ButtonState::Released => 
                    {
                        self.mouse_buttons_pressed.remove(& button);
                    }
                },
                mouse::Event::WheelScrolled { delta_x: _, delta_y } => 
                {
                    self.mouse_scroll_wheel = coffee::graphics::Point::new(0.0, delta_y);
                },
                _ => {}
            },
            Event::Keyboard(keyboard_event) => match keyboard_event 
            {
                keyboard::Event::Input { key_code, state } => match state 
                {
                    ButtonState::Pressed => 
                    {
                        self.keys_pressed.insert(key_code);
                    },
                    ButtonState::Released => 
                    {
                        self.keys_pressed.remove(& key_code);
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
}

///
/// Keeps track of the window's dimensions; necessary to create responsive states.
///
/// The window size provides some convenience methods related to state calculations.
/// The UI buttons are constant size 
///
#[derive(Clone, Copy, Debug)]
pub struct WindowSize 
{
    width: f32,
    height: f32
}

impl WindowSize 
{
    ///
    /// Returns the bottom-left corner of the board.
    ///
    pub fn get_board_corner (& self) -> coffee::graphics::Point 
    {
        let game_area_w = match self.is_portrait()
        {
            true  => self.get_tile_size() * 10.0,
            false => self.get_tile_size() * 10.0 
        };
        let game_area_h = match self.is_portrait()
        {
            true  => self.get_tile_size() * 10.0,
            false => self.get_tile_size() * 10.0
        };

        let x = (self.width - game_area_w) / 2.0;
        let y = (self.height - self.get_button_height() - 2.0 * self.get_spacer() - game_area_h) / 2.0;

        coffee::graphics::Point::new(x, self.get_button_height() + self.get_spacer() + y)
    }

    ///
    /// Returns the side length of the board.
    ///
    pub fn get_board_size (& self) -> f32 
    {
        10.0 * self.get_tile_size()
    }

    ///
    /// Returns the width of the border.
    ///
    pub fn get_border_width (& self) -> f32 
    {
        0.05 * self.get_tile_size()
    }

    ///
    /// Returns the height of the button bar.
    ///
    pub fn get_button_height (& self) -> f32 
    {
        0.05 * self.height
    }

    ///
    /// Returns the divisor for the game area.
    ///
    pub fn get_divisor (& self) -> f32 
    {
        match self.is_portrait()
        {
            true  => 10.0,
            false => 10.0
        }
    }

    pub fn get_spacer (& self) -> f32 
    {
        0.05 * self.height
    }
        
    ///
    /// Returns the side length of a tile.
    ///
    pub fn get_tile_size (& self) -> f32 
    {
        let num_tiles_w = match self.is_portrait()
        {
            true  => 10.0,
            false => 10.0
        };
        let num_tiles_h = match self.is_portrait()
        {
            true  => 10.0,
            false => 10.0
        };
        let size_w = (self.width - 2.0 * self.get_spacer()) / num_tiles_w;
        let size_h = (self.height - 2.0 * self.get_spacer() - self.get_button_height()) / num_tiles_h;
        
        size_w.min(size_h)
    }

    ///
    /// Determines whether this is portrait mode.
    ///
    pub fn is_portrait (& self) -> bool 
    {
        self.width < self.height
    }

    ///
    /// Grabs the window dimensions from the window.
    ///
    pub fn new (width: f32, height: f32) -> WindowSize 
    {
        WindowSize { width, height }
    }
}

///
/// An enum describing the events produced by buttons.
///
#[derive(Clone, Copy, Debug)]
pub enum EventState 
{
    NewGameButton,
    SetupModeButton,
    CancelSetupButton,
    ConfirmSetupButton,
    PlayMoveButton,
    CancelSearchButton,
    UndoMoveButton
}

