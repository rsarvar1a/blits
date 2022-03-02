
use lits::{Board, Tetromino};

///
/// An alias on usize for readability.
///
pub type NodeID = usize;

///
/// An alias on usize for readability.
///
pub type MoveID = usize;

///
/// An outcome for the player in the tree scope.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome 
{
    Win,
    Loss
}

impl std::cmp::Ord for Outcome 
{
    fn cmp (& self, other: & Outcome) -> std::cmp::Ordering 
    {
        match (self, other)
        {
            (Outcome::Win, Outcome::Loss) => std::cmp::Ordering::Greater,
            (Outcome::Loss, Outcome::Win) => std::cmp::Ordering::Less,
            _                             => std::cmp::Ordering::Equal
        }
    }
}

impl std::cmp::PartialOrd for Outcome 
{
    fn partial_cmp(& self, other: & Outcome) -> Option<std::cmp::Ordering> 
    {
        Some(self.cmp(other))
    }
}

impl std::convert::From<f32> for Outcome 
{
    fn from (value: f32) -> Outcome 
    {
        if value > 0.0
        {
            Outcome::Win 
        }
        else 
        {
            Outcome::Loss
        }
    }
}

impl Outcome 
{
    ///
    /// Inverts the outcome.
    ///
    pub fn next (& self) -> Outcome 
    {
        match self 
        {
            Outcome::Win  => Outcome::Loss,
            Outcome::Loss => Outcome::Win
        }
    }

    ///
    /// Gets the unit value of this outcome in the primary perspective.
    ///
    pub fn value (& self) -> f32 
    {
        match self 
        {
            Outcome::Win => 1.0,
            _            => 0.0
        }
    }
}

///
/// Represents a state in a gametree, with the corresponding in-action that lead to this state from
/// its parent.
///
pub struct Node 
{
    pub id: NodeID,
    pub parent: Option<NodeID>,
    pub oldest_child: NodeID,
    pub num_children: usize,

    pub state: Board,
    pub in_action: MoveID,
    pub outcome: Option<Outcome>,

    pub n: f32,
    pub p: f32, 
    pub v: f32
}

impl Node 
{
    ///
    /// Returns the action that took this node's parent to this node.
    ///
    pub fn action (& self) -> Tetromino 
    {
        (self.in_action as usize).into()
    }

    ///
    /// Determines whether this node is known to have a terminal state.
    ///
    pub fn is_unsolved (& self) -> bool 
    {
        self.outcome.is_none() 
    }

    ///
    /// Determines whether this node is unvisited.
    ///
    pub fn is_unvisited (& self) -> bool
    {
       self.is_unsolved() && ! self.is_visited() 
    }

    ///
    /// Determines whether this node is strictly visited.
    ///
    pub fn is_visited (& self) -> bool 
    {
        self.num_children > 0
    }

    ///
    /// Creates a new node representing an unvisited parent-action-child state tuple.
    ///
    pub fn new (id: NodeID, parent: Option<NodeID>, state: & Board, outcome: Option<Outcome>, in_action: MoveID, p: f32) -> Node
    {
        Node 
        {
            id,
            parent,
            oldest_child: 0,
            num_children: 0,

            state: state.clone(),
            in_action,
            outcome,

            n: 0.0,
            p,
            v: 0.0
        }
    }

    ///
    /// Denotes that this node has a solved playout result.
    ///
    pub fn solve (& mut self, outcome: Outcome)
    {
        self.outcome = Some(outcome);
    }

    ///
    /// Updates this node's stats.
    ///
    pub fn update (& mut self, value: f32, visits: f32)
    {
        self.n += visits;
        self.v += value;
    }

    ///
    /// Updates this node's stats by way of overwriting.
    ///
    pub fn update_overwrite (& mut self, new_val: f32, new_vis: f32)
    {
        self.n = new_vis;
        self.v = new_val;
    }

    ///
    /// Sets a new probability on this node.
    ///
    pub fn update_prob (& mut self, prob: f32)
    {
        self.p = prob;
    }

    ///
    /// Visits this node.
    ///
    pub fn visit (& mut self, oldest_child: NodeID, num_children: usize)
    {
        self.oldest_child = oldest_child;
        self.num_children = num_children;
    }
}

