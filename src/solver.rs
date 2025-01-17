//! A solver that takes observations and derives information about the atom grid.

use crate::atom_grid::GRID_SIZE;
use crate::i8vec2::I8Vec2;
use crate::laser::Direction::{Down, Left, Right, Up};
use crate::laser::LaserTip;
use crate::observation::{Observations, LASER_ABSORBED, LASER_REFLECTED};
use crate::solver::GridKnowledge::{Empty, Unknown};
use std::fmt::Write;
use GridKnowledge::Atom;

#[derive(Default)]
pub struct UncertainGrid {
    atoms: [[GridKnowledge; GRID_SIZE]; GRID_SIZE],
}

impl UncertainGrid {
    fn get(&self, v: I8Vec2) -> GridKnowledge {
        self.atoms[v.x as usize][v.y as usize]
    }

    /// Sets a value, but does nothing if the given vector is outside the grid.
    fn set_safe(&mut self, v: I8Vec2, knowledge: GridKnowledge) {
        if knowledge == Unknown {
            panic!("Can not update with Unknown at {:?}", v);
        }
        if v.in_grid() {
            // Check consistency and crash when updating with inconsistent information.
            let previous_knowledge = self.atoms[v.x as usize][v.y as usize];
            if previous_knowledge != Unknown && previous_knowledge != knowledge {
                panic!(
                    "Updating existing knowledge {:?} with inconsistent {:?} at {:?}",
                    previous_knowledge, knowledge, v
                );
            }
            self.atoms[v.x as usize][v.y as usize] = knowledge;
        }
    }
}

pub fn draw(grid: &UncertainGrid, observations: &Observations) -> Result<String, std::fmt::Error> {
    let mut f = String::new();
    // first, display the row above with lasers pointing down
    f.write_str("  ")?;
    for obs in observations.sides[Down as usize] {
        f.write_str(&format!(" {}", obs))?;
    }
    f.write_char('\n')?;

    let left_border = observations.sides[Right as usize];
    let right_border = observations.sides[Left as usize];

    // Show rows
    for y in 0..GRID_SIZE {
        let left_obs = left_border[y];
        let right_obs = right_border[y];

        f.write_str(&format!(" {}", left_obs))?;
        for x in 0..GRID_SIZE {
            match grid.get(I8Vec2::new(x as i8, y as i8)) {
                Unknown => f.write_str(" ?")?,
                Atom => f.write_str(" o")?,
                Empty => f.write_str(" .")?,
            }
        }
        f.write_str(&format!(" {}\n", right_obs))?;
    }

    f.write_str("  ")?;
    for obs in observations.sides[Up as usize] {
        f.write_str(&format!(" {}", obs))?;
    }
    f.write_char('\n')?;

    Ok(f)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum GridKnowledge {
    Unknown,
    Atom,
    Empty,
}

impl Default for GridKnowledge {
    fn default() -> Self {
        Unknown
    }
}

pub fn solve_as_much_as_you_can(observations: &Observations) -> UncertainGrid {
    let mut grid = UncertainGrid::default();

    letter_finds_four_empty_spaces(&mut grid, observations);
    reflection_is_not_blocked(&mut grid, observations);

    // Benefits from "Free field" information.
    absorption_with_one_free_field(&mut grid, observations);

    grid
}

fn reflection_is_not_blocked(grid: &mut UncertainGrid, observations: &Observations) {
    for (direction, shift, obs) in observations.iter() {
        if obs == LASER_REFLECTED {
            let l = LaserTip::new(shift as u8, direction);
            let center = l.forward().position();

            grid.set_safe(center, Empty);
        }
    }
}

fn absorption_with_one_free_field(grid: &mut UncertainGrid, observations: &Observations) {
    for (direction, shift, obs) in observations.iter() {
        if obs == LASER_ABSORBED {
            let l = LaserTip::new(shift as u8, direction);
            let center = l.forward().position();

            if grid.get(center) == Empty {
                grid.set_safe(center + direction.clockwise().dxy(), Empty);
                grid.set_safe(center + direction.counter_clockwise().dxy(), Empty);
            }
        }
    }
}

fn letter_finds_four_empty_spaces(grid: &mut UncertainGrid, observations: &Observations) {
    for (direction, shift, obs) in observations.iter() {
        if obs.is_letter() {
            let l = LaserTip::new(shift as u8, direction);
            let center = l.forward().position();

            grid.set_safe(center, Empty);
            grid.set_safe(center + I8Vec2::new(0, 1), Empty);
            grid.set_safe(center + I8Vec2::new(0, -1), Empty);
            grid.set_safe(center + I8Vec2::new(1, 0), Empty);
            grid.set_safe(center + I8Vec2::new(-1, 0), Empty);
        }
    }
}
