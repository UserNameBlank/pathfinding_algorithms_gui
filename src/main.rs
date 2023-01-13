#![windows_subsystem = "windows"]

extern crate glutin_window;
extern crate graphics;
extern crate piston;
extern crate opengl_graphics;

use opengl_graphics::{GlGraphics, OpenGL};
use glutin_window::GlutinWindow as Window;
use graphics::types::{Color, Scalar};
use piston::{Button, Events, EventSettings, MouseButton, MouseCursorEvent, PressEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, WindowSettings};
use crate::astar::Node;

mod astar;

const WINDOW_SIZE: [u32; 2] = [600, 600];
const AMOUNT_OF_CELLS_ROW: u32 = 20;
const AMOUNT_OF_CELLS: u32 = AMOUNT_OF_CELLS_ROW * AMOUNT_OF_CELLS_ROW;
const CELL_SIZE: Scalar = WINDOW_SIZE[0] as Scalar / AMOUNT_OF_CELLS_ROW as Scalar;
const CELL_RADIUS: Scalar = CELL_SIZE / 2.;

const BACKGROUND_COLOR: Color = [0.2, 0.2, 0.2, 1.];
const NORMAL_CELL_COLOR: Color = [0.9, 0.9, 0.9, 1.];
const START_CELL_COLOR: Color = [0.1, 0.7, 1., 1.];
const TARGET_CELL_COLOR: Color = [0.1, 0.7, 1., 1.];
const SOLID_CELL_COLOR: Color = [0., 0., 0., 1.];
const HOVERED_CELL_COLOR: Color = [0.6, 0.6, 0.6, 0.5];
const PATH_CELL_COLOR: Color = [0.0, 0.4, 0.8, 1.];
const OPENED_CELL_COLOR: Color = [0.0, 1.0, 0.1, 1.];
const CLOSED_CELL_COLOR: Color = [1.0, 0.1, 0.0, 1.];

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position(u16, u16);
impl Position {
    fn new(x: u16, y: u16) -> Position {
        Position(x, y)
    }
}

#[derive(Clone, Copy)]
pub enum CellState {
    Solid,
    Normal,
    Path,
    Opened,
    Closed
}

struct Game {
    gl: GlGraphics,
    target_pos: Position,
    start_pos: Position,
    states: Vec<CellState>,
    hovered_cell: Position,

    is_finding: bool,
    start_node: Node,
    end_node: Node,
    opened: Vec<Node>,
    closed: Vec<Node>,
}
impl Game {
    fn new(gl: GlGraphics) -> Game {
        Game {
            gl,
            target_pos: Position::new(6, 4),
            start_pos: Position::new(17, 14),
            states: vec![CellState::Normal; AMOUNT_OF_CELLS as usize],
            hovered_cell: Position::new(0, 0),
            is_finding: false,
            start_node: {
                let diff = {
                    let distance_to_target_vec = Position::new(
                        (6 as i32 - 17 as i32) as u16,
                        (4 as i32 - 14 as i32) as u16
                    );
                    ((distance_to_target_vec.0 as f32 * distance_to_target_vec.0 as f32) + (distance_to_target_vec.1 as f32 * distance_to_target_vec.1 as f32)).sqrt() as u32
                };
                Node::new(Position::new(17, 14), diff, 0)
            },
            end_node: Node::new(Position::new(6, 4), 0, u32::MAX),
            opened: vec![],
            closed: vec![]
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |c, gl| {
            gl.clear_color(BACKGROUND_COLOR);

            let cell = rectangle::centered_square(0., 0., CELL_RADIUS - 1.);

            let transform = c.transform.trans(CELL_RADIUS, CELL_RADIUS);

            for x in 0..AMOUNT_OF_CELLS_ROW {
                for y in 0..AMOUNT_OF_CELLS_ROW {
                    let color = match self.states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] {
                        CellState::Normal => NORMAL_CELL_COLOR,
                        CellState::Solid => SOLID_CELL_COLOR,
                        CellState::Path => PATH_CELL_COLOR,
                        CellState::Opened => OPENED_CELL_COLOR,
                        CellState::Closed => CLOSED_CELL_COLOR
                    };

                    let x = x as Scalar * CELL_SIZE;
                    let y = y as Scalar * CELL_SIZE;
                    let transform = transform.trans(x, y);

                    rectangle(color, cell, transform, gl);
                }
            }

            {
                let transform = transform.trans(self.start_pos.0 as Scalar * CELL_SIZE, self.start_pos.1 as Scalar * CELL_SIZE);
                rectangle(START_CELL_COLOR, cell, transform, gl);
            }
            {
                let transform = transform.trans(self.target_pos.0 as Scalar * CELL_SIZE, self.target_pos.1 as Scalar * CELL_SIZE);
                rectangle(TARGET_CELL_COLOR, cell, transform, gl);
            }
            {
                let transform = transform.trans(self.hovered_cell.0 as Scalar * CELL_SIZE, self.hovered_cell.1 as Scalar * CELL_SIZE);
                rectangle(HOVERED_CELL_COLOR, cell, transform, gl);
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        if self.is_finding {
            let res = astar::update_path_finding(self.start_node, self.end_node, &mut self.states, &mut self.opened, &mut self.closed);
            self.is_finding = !res;
        }
    }

    fn mouse_hovered(&mut self, pos: [f64; 2]) {
        let x = (pos[0] / CELL_SIZE as f64) as u16;
        let y = (pos[1] / CELL_SIZE as f64) as u16;

        self.hovered_cell = Position(x, y);
    }

    fn mouse_button_pressed(&mut self, btn: MouseButton) {
        if let MouseButton::Left = btn {
            let Position(x, y ) = self.hovered_cell;
            let x = x as u32;
            let y = y as u32;

            let state = self.states[(y * AMOUNT_OF_CELLS_ROW + x) as usize];
            match state {
                CellState::Normal => self.states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Solid,
                CellState::Solid => self.states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Normal,
                _ => self.states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Solid
            }
        } else if let MouseButton::Right = btn {
            if !self.is_finding {
                astar::start_path_finding(self.start_node, &mut self.opened, &mut self.closed);
                self.is_finding = true;
            }
        } else if let MouseButton::Middle = btn {
            for state in &mut self.states {
                *state = CellState::Normal;
            }
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("a*-algorithm test", WINDOW_SIZE)
        .resizable(false)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game::new(GlGraphics::new(opengl));

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(args) = e.update_args() {
            game.update(&args);
        }

        if let Some(Button::Mouse(mouse)) = e.press_args() {
            game.mouse_button_pressed(mouse);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            game.mouse_hovered(pos);
        }
    }
}
