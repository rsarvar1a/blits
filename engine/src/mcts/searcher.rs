
use crate::config::*;

use lits::{Board, Player, Tetromino};

use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::node::*;
use super::sync::*;
use super::threadpool::*;

///
/// An alias on usize for readability.
///
pub type TreeID = usize;

///
/// An enum describing the type of event to wait for.
///
#[derive(Clone, Copy, Debug)]
pub enum SearcherEvent 
{
    Start,
    Finish
}

impl std::convert::Into<bool> for SearcherEvent 
{
    fn into (self) -> bool 
    {
        match self 
        {
            SearcherEvent::Start  => true,
            SearcherEvent::Finish => false
        }
    }
}

///
/// An enum describing the type of a searcher.
///
pub enum SearcherFilter
{
    Main,
    Worker,
    All
}

impl SearcherFilter 
{
    ///
    /// Determines whether the given filter matches the given thread. 
    ///
    pub fn matches (& self, id: TreeID) -> bool
    {
        match self 
        {
            SearcherFilter::Main   => id == 0,
            SearcherFilter::Worker => id != 0,
            SearcherFilter::All    => true
        }
    }
}

///
/// Gives the thread manager an unsafe view into the searcher 
/// for synchronization purposes.
///
pub struct SearcherHandle 
{
    pub ptr: UnsafeCell<* mut Searcher>
}

unsafe impl Sync for SearcherHandle {}
unsafe impl Send for SearcherHandle {}

///
/// A single searcher that lives on a particular work thread.
///
pub struct Searcher 
{
    pub pool: * mut ThreadPool,
    pub config: MCTSConfig, 

    pub id: TreeID,
    pub kill: AtomicBool,
    pub search_status: Arc<Guard>,
    pub cond_variable: Arc<Latch>,

    pub state: Board,
    pub solve_for: Player,

    pub tree: Vec<Node>,
    pub root: NodeID,

    pub best_move: MoveID,
    pub best_eval: f32
}

unsafe impl Sync for Searcher {}
unsafe impl Send for Searcher {}

impl Searcher 
{
    pub fn backpropagate (& mut self, leaf: NodeID, value: f32, has_solution: bool)
    {
        let mut id = leaf;
        let mut val = value;
        let mut has_sol = has_solution;
        let discount = self.config.discount;

        loop 
        {
            if has_sol && self.node_immut(id).is_unsolved()
            {
                let mut all = true;
                let mut worst_outcome = None;

                for child in self.children_of_immut(id)
                {
                    if child.is_unvisited() || child.is_unsolved()
                    {
                        all = false;
                    }
                    else if worst_outcome.is_none() || child.outcome < worst_outcome
                    {
                        worst_outcome = child.outcome;
                    }
                }

                let node = self.node(id);

                if worst_outcome == Some(Outcome::Loss)
                {
                    node.solve(Outcome::Win);
                    val = - node.v + node.n + 1.0;
                }
                else if node.is_visited() && all 
                {
                    let best_in_pov = worst_outcome.unwrap().next();
                    node.solve(best_in_pov);
                    val = - node.v - node.n - 1.0;
                }
                else 
                {
                    has_sol = false;
                }
            }

            let node = self.node(id);

            node.v += val;
            node.n += 1.0;

            if node.parent.is_none()
            {
                break;
            }

            val = - discount * val;
            id = node.parent.unwrap();
        }
    }

    ///
    /// Extracts the best A-V pair from the children of the root position.
    ///
    pub fn best_av_pair (& mut self) -> (MoveID, f32)
    {
        if ! self.state.has_moves()
        {
            return (0, f32::NEG_INFINITY);
        }

        let root = self.root;

        let mut best_move = 0;
        let mut best_score = f32::NEG_INFINITY;

        for child in self.children_of(root) 
        {
            let score = match child.outcome
            {
                Some(Outcome::Win)  => f32::NEG_INFINITY,
                Some(Outcome::Loss) => f32::INFINITY,
                None                => - child.v / child.n.max(1.0)
            };

            if score > best_score
            {
                best_score = score;
                best_move = child.action().into();
            }
        }

        self.best_move = best_move;
        self.best_eval = best_score;

        (best_move, best_score)
    }

    ///
    /// Returns the children of the given node.
    ///
    pub fn children_of (& mut self, node: NodeID) -> & mut [Node]
    {
        let node  = & self.tree[node];
        let start = node.oldest_child;
        let end   = start + node.num_children;

        & mut self.tree[start .. end]
    }

    ///
    /// Returns the children of the given node.
    ///
    pub fn children_of_immut (& self, node: NodeID) -> & [Node]
    {
        let node  = & self.tree[node];
        let start = node.oldest_child;
        let end   = start + node.num_children;

        & self.tree[start .. end]
    }

    ///
    /// Clears this searcher, required before successive calls to search() on the MCTS instance.
    ///
    pub fn clear (& mut self)
    {
        self.tree = Vec::new();
        self.root = 0;

        self.state = Board::blank();

        self.best_move = 0;
        self.best_eval = 0.0;
    }

    ///
    /// Gets the best continuation.
    ///
    pub fn continuation (& self, id: NodeID) -> NodeID 
    {
        let mut best_id = None;
        let mut best_score = f32::NEG_INFINITY;

        for child in self.children_of_immut(id)
        {
            let q = self.get_q(id, child.id);
            let u = self.get_u(id, child.id);
            let score = q + u;
            if score > best_score
            {
                best_id = Some(child.id);
                best_score = score;
            }
        }

        best_id.unwrap()
    }

    ///
    /// Gets the q value, or the exploitation value of the given states-action pair.
    ///
    pub fn get_q (& self, parent: NodeID, child: NodeID) -> f32 
    {
        let parent = self.node_immut(parent);
        let child = self.node_immut(child);

        if let Some(outcome) = child.outcome
        {
            outcome.next().value()
        }
        else if child.num_children == 0 
        {
            parent.v / parent.n
        }
        else 
        {
            - child.v / child.n
        }
    }

    ///
    /// Gets the u value, or the exploration value of the given states-action pair.
    ///
    pub fn get_u (& self, parent: NodeID, child: NodeID) -> f32 
    {
        let parent = self.node_immut(parent);
        let child = self.node_immut(child);

        let visits = parent.n.sqrt();
        self.config.uct_const * child.p * visits / (1.0 + child.n)
    }

    ///
    /// Idles, waiting for the pool to unlock.
    ///
    pub fn idle (& mut self) 
    {
        self.search_status.set(false);
        loop 
        {
            self.cond_variable.wait();
            if self.kill.load(Ordering::SeqCst)
            {
                return;
            }
            self.launch();
        }
    }

    ///
    /// Initializes this search tree with the given position and 
    /// optimizing subject.
    ///
    pub fn initialize (& mut self, position: & Board)
    {
        self.clear();

        self.state = position.clone();
        self.solve_for = position.to_move();

        self.tree.push(Node::new(0, None, position, None, Tetromino::null().into(), 0.0));
        self.root = 0;
    }

    ///
    /// Determines if this thread is main.
    ///
    pub fn is_main (& self) -> bool 
    {
        self.id == 0
    }

    ///
    /// Starts this searcher.
    ///
    pub fn launch (& mut self)
    {
        self.search_status.set(true);

        match self.is_main()
        {
            true  => self.launch_main(),
            false => self.search_root()
        };

        self.search_status.set(false);
    }

    ///
    /// Starts the search on the main thread, which has 
    /// the specific responsbility to collect the best 
    /// move in the position.
    ///
    pub fn launch_main (& mut self) 
    {
        self.pool().work_cond.set();

        self.search_root();

        self.pool().work_cond.lock();
        self.pool().set_stop_requirement(true);
        self.pool().wait_for(SearcherFilter::Worker, SearcherEvent::Finish);

        let mut best_move  = 0;
        let mut best_score = f32::NEG_INFINITY;

        self.pool().threads.iter_mut()
            .map(|handle| unsafe { & mut (** handle.get()) })
            .for_each(
                |thread|
                {
                    let (this_piece, this_score) = thread.best_av_pair().clone();
                    if this_score > best_score 
                    {
                        best_score = this_score;
                        best_move  = this_piece;
                    }
                }
            );

        self.best_move = best_move;
        self.best_eval = best_score;
    }

    ///
    /// Returns a new searcher.
    ///
    pub fn new (pool: * mut ThreadPool, config: MCTSConfig, id: TreeID, cond_variable: Arc<Latch>) -> Searcher
    {
        Searcher 
        {
            pool,
            config,

            id,
            kill: AtomicBool::new(false),
            search_status: Arc::new(Guard::new(true)),
            cond_variable,

            state: Board::blank(),
            solve_for: Player::None,

            tree: Vec::new(),
            root: 0,

            best_move: 0,
            best_eval: 0.0
        }
    }

    ///
    /// Returns the node with the given id.
    ///
    pub fn node (& mut self, id: NodeID) -> & mut Node 
    {
        & mut self.tree[id]
    }

    ///
    /// Returns the node with the given id.
    ///
    pub fn node_immut (& self, id: NodeID) -> & Node 
    {
        & self.tree[id]
    }

    ///
    /// Returns the threadpool from this searcher's parent 
    /// in a somewhat horrifying way.
    ///
    pub fn pool (& self) -> & mut ThreadPool
    {
        unsafe 
        { 
            & mut (* self.pool) 
        }
    }

    ///
    /// Returns the root.
    ///
    pub fn root (& mut self) -> & mut Node 
    {
        & mut self.tree[self.root]
    }

    ///
    /// Starts the search from this searcher's root.
    ///
    pub fn search_root (& mut self)
    {
        let allowed_duration = Duration::from_millis(self.config.max_time_ms as u64);
        let start = Instant::now();
        let mut end = Instant::now();

        while ! self.stop() && (end - start) < allowed_duration && self.root().is_unsolved()
        {
            end = Instant::now();
            let mut id = self.root;

            loop 
            {
                let node = self.node(id);
                if let Some(outcome) = node.outcome 
                {
                    self.backpropagate(id, outcome.value(), true);
                    break;
                }
                else if node.is_unvisited()
                {
                    let (value, found_leaf) = self.visit(id);
                    self.backpropagate(id, value, found_leaf);
                }
                else 
                {
                    id = self.continuation(id);
                }
            }
        }
    }

    ///
    /// Determines whether to stop.
    ///
    pub fn stop (& mut self) -> bool 
    {
        self.pool().stop.load(Ordering::Relaxed)
    }

    ///
    /// Visits the given node, expanding it if necessary, and returns its value 
    /// as well as whether the position is solved in this subtree.
    ///
    pub fn visit (& mut self, id: NodeID) -> (f32, bool)
    {
        let insertion_point = self.tree.len();
        let node = self.node_immut(id);
        let game = node.state.clone();
        let (policy, value) = self.pool().parent().policy().predict(& game);
        let mut num_children = 0;
        let mut any = false;
        let mut max_action = f32::NEG_INFINITY;

        // Add a new node for every possible move.

        for tetromino in & game.enumerate_moves()
        {
            let mut next_state = game.clone();
            next_state.place_tetromino(& tetromino);
            let over = ! next_state.has_moves();
            let outcome = match over 
            {
                // Transform the score to be in the player to move's perspective.
                // If it is positive, this player won, otherwise they lost.
                
                true =>
                {
                    any = true;
                    Some(<Outcome as From<f32>>::from(game.score() as f32 * game.to_move().value() as f32))
                },
                false => None
            };
            let action : usize = <Tetromino as Into<usize>>::into(tetromino.clone());
            let pred = policy[action];
            max_action = max_action.max(pred);
            let child = Node::new(self.tree.len(), Some(id), & next_state, outcome, action, pred);
            self.tree.push(child);
            num_children += 1;
        }

        // Mark this node as visited, linking its children references into the tree.

        let node = self.node(id);
        node.visit(insertion_point, num_children);

        // Softmax its children.

        let mut total = 0.0;
        for child in self.children_of(id)
        {
            child.p = (child.p - max_action).exp();
            total += child.p;
        }
        for child in self.children_of(id)
        {
            child.p /= total;
        }

        (value, any)
    }
}
