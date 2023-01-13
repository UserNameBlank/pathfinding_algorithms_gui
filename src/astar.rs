use std::fmt::Error;
use crate::{AMOUNT_OF_CELLS_ROW, CellState, Position};

fn manhattan(pa: Position, pb: Position) -> u32 {
    ((pb.0 as isize - pa.0 as isize).abs() + (pb.1 as isize - pa.1 as isize).abs()) as u32
}

fn tchebychev(pa: Position, pb: Position) -> u32 {
    std::cmp::max((pb.0 as isize - pa.0 as isize).abs(), (pb.1 as isize - pa.1 as isize).abs()) as u32
}

#[derive(Clone, Copy)]
pub struct Node {
    parent: Option<u32>,
    pos: Position,
    g_cost: u32,
    h_cost: u32,
    f_cost: u32
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}
impl Node {
    pub fn new(pos: Position, gcost: u32, hcost: u32) -> Node {
        Node {
            parent: None,
            pos,
            g_cost: gcost,
            h_cost: hcost,
            f_cost: gcost + hcost
        }
    }

    fn compute_costs(&mut self, target: Node, nodes: &Vec<Node>) -> (u32, u32, u32) {
        let h_cost = {
            if let Some(index) = self.parent {
                if self.pos.1 != nodes[index as usize].pos.1 && self.pos.0 != nodes[index as usize].pos.0 {
                    nodes[index as usize].h_cost + 14
                } else {
                    nodes[index as usize].h_cost + 1
                }
            } else {
                0
            }
        };
        /*{
            let distance_to_target_vec = Position::new(
                (target.pos.0 as i32 - self.pos.0 as i32) as u16,
                (target.pos.1 as i32 - self.pos.1 as i32) as u16
            );
            ((distance_to_target_vec.0 as f32 * distance_to_target_vec.0 as f32) + (distance_to_target_vec.1 as f32 * distance_to_target_vec.1 as f32)).sqrt() as u32
        }*/
        let g_cost = tchebychev(self.pos, target.pos) * 10;

        let f_cost = g_cost + h_cost;

        self.g_cost = g_cost;
        self.h_cost = h_cost;
        self.f_cost = f_cost;

        (g_cost, h_cost, f_cost)
    }

    fn computed(mut self, target: Node, nodes: &Vec<Node>) -> Self {
        self.compute_costs(target, nodes);
        self
    }

    fn get_neighbours(&self, start: Node, nodes: &Vec<Node>) -> Vec<Node> {
        let mut neighbours: Vec<Node> = vec![];
        for i in [-1, 0, 1] {
            for j in [-1, 0, 1] {
                if i == 0 && j == 0 { continue; }

                let mut node = self.clone();
                node.pos.0 = (node.pos.0 as i32 + i) as u16;
                node.pos.1 = (node.pos.1 as i32 + j) as u16;
                if node.pos.0 >= 0 && node.pos.0 < AMOUNT_OF_CELLS_ROW as u16 {
                    if node.pos.1 >= 0u16 && node.pos.1 < AMOUNT_OF_CELLS_ROW as u16 {
                        neighbours.push(node.computed(start, nodes));
                    }
                }
            }
        }
        neighbours
    }

    fn get_neighbours_ovh(&self, start: Node, nodes: &Vec<Node>) -> Vec<Node> {
        let mut neighbours: Vec<Node> = vec![];
        for i in [-1, 1] {
            let mut node = self.clone();
            node.pos.0 = (node.pos.0 as i32 + i) as u16;
            node.pos.1 = (node.pos.1 as i32) as u16;
            if node.pos.0 >= 0 && node.pos.0 < AMOUNT_OF_CELLS_ROW as u16 {
                neighbours.push(node.computed(start, nodes));
            }
        }
        for i in [-1, 1] {
            let mut node = self.clone();
            node.pos.0 = (node.pos.0 as i32) as u16;
            node.pos.1 = (node.pos.1 as i32 + i) as u16;
            if node.pos.1 >= 0 && node.pos.1 < AMOUNT_OF_CELLS_ROW as u16 {
                neighbours.push(node.computed(start, nodes));
            }
        }
        neighbours
    }

    fn is_traversable(&self, states: &Vec<CellState>) -> bool {
        let x = self.pos.0 as u32;
        let y = self.pos.1 as u32;
        let state = if let Some(state) = states.get((y * AMOUNT_OF_CELLS_ROW + x) as usize) { state } else { return false; };

        if let CellState::Solid = state {
            return false;
        }
        true
    }
}

fn get_path_nodes(node: &Node, nodes: &Vec<Node>) -> Vec<Node> {
    let mut current = node;
    let mut path = vec![current.clone()];

    while let Some(index) = current.parent {
        current = &nodes[index as usize];
        path.push(current.clone());
    }

    path.reverse();
    path
}

pub fn find_path(start_node: Node, end_node: Node, states: &mut Vec<CellState>) {
    let mut opened: Vec<Node> = vec![start_node];
    let mut closed: Vec<Node> = vec![];

    loop {
        let current = opened.iter().min_by_key(|n| n.f_cost).expect("No Path was found at all.").clone();
        opened.retain(|n| n.pos != current.pos);
        closed.push(current);

        // Set State
        let x = current.pos.0 as u32;
        let y = current.pos.1 as u32;
        states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Closed;

        if current == end_node {
            let result = get_path_nodes(&current, &closed);
            for node in result {
                println!("{:?}", node.pos);

                let x = node.pos.0 as u32;
                let y = node.pos.1 as u32;
                states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Path;
            }
            return;
        }

        for mut neighbour in current.get_neighbours(start_node, &closed) {
            if !neighbour.is_traversable(states) || closed.contains(&neighbour) {
                continue;
            }

            if neighbour.f_cost < current.f_cost || !opened.contains(&neighbour) {
                neighbour.parent = Some(closed.len() as u32 - 1);
                neighbour.compute_costs(end_node, &closed);
                if !opened.contains(&neighbour) {
                    opened.push(neighbour);

                    // Set State
                    let x = neighbour.pos.0 as u32;
                    let y = neighbour.pos.1 as u32;
                    states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Opened;
                }
            }
        }
    }
}

pub fn start_path_finding(start_node: Node, opened: &mut Vec<Node>, closed: &mut Vec<Node>) {
    opened.clear();
    closed.clear();
    opened.push(start_node);
}

pub fn update_path_finding(start_node: Node, end_node: Node, states: &mut Vec<CellState>, opened: &mut Vec<Node>, closed: &mut Vec<Node>) -> bool {
    let current = opened.iter().min_by_key(|n| n.f_cost).expect("No Path was found at all.").clone();
    opened.retain(|n| n.pos != current.pos);
    closed.push(current);

    // Set State
    let x = current.pos.0 as u32;
    let y = current.pos.1 as u32;
    states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Closed;

    if current == end_node {
        let result = get_path_nodes(&current, &closed);
        for node in result {
            println!("{:?}", node.pos);

            let x = node.pos.0 as u32;
            let y = node.pos.1 as u32;
            states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Path;
        }
        return true;
    }

    for mut neighbour in current.get_neighbours_ovh(start_node, &closed) {
        if !neighbour.is_traversable(states) || closed.contains(&neighbour) {
            continue;
        }

        if neighbour.f_cost < current.f_cost || !opened.contains(&neighbour) {
            neighbour.parent = Some(closed.len() as u32 - 1);
            neighbour.compute_costs(end_node, &closed);
            if !opened.contains(&neighbour) {
                opened.push(neighbour);

                // Set State
                let x = neighbour.pos.0 as u32;
                let y = neighbour.pos.1 as u32;
                states[(y * AMOUNT_OF_CELLS_ROW + x) as usize] = CellState::Opened;
            }
        }
    }
    false
}