
#![allow(mutable_transmutes)]

use lits::Board;

use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use super::node::MoveID;
use super::mcts::*;
use super::searcher::*;
use super::sync::*;

use tabled::{Table, Tabled};

use utils::*;
use utils::notate::Notate;

///
/// A stats object that is printed into the summary table.
///
#[derive(Clone, Debug, Tabled, PartialEq)]
pub struct SearcherStats
{
    pub tetromino: String,
    pub visits: f32,
    pub prob: f32,
    pub eval: f32,
    pub components: i32
}

impl std::cmp::PartialOrd for SearcherStats
{
    fn partial_cmp (& self, other: & SearcherStats) -> Option<std::cmp::Ordering>
    {
        self.eval.partial_cmp(& other.eval)
    }
}

///
/// The resource manager for the threads that make up an MCTS search pool.
///
pub struct ThreadPool
{
    pub mcts: * const MCTS,
    pub state: Board,

    pub threads: Vec<UnsafeCell<* mut Searcher>>,
    pub handles: Vec<JoinHandle<()>>,
    
    pub cond: Arc<Latch>,
    pub stop: AtomicBool,
}

impl ThreadPool 
{
    ///
    /// Attaches a new thread to the thread pool and starts it.
    ///
    pub fn attach_one (& mut self)
    {
        unsafe 
        {
            let mem_layout = Layout::new::<Searcher>();
            let result = alloc_zeroed(mem_layout);
            let searcher_ptr : * mut Searcher = result.cast() as * mut Searcher;

            let searcher_id = self.threads.len();
            let cond_variable = self.cond.clone();
            let pool : * mut ThreadPool = self;
            let config = (* self.mcts).config();

            ptr::write(searcher_ptr, Searcher::new(pool, config, searcher_id, cond_variable));

            self.threads.push(UnsafeCell::new(searcher_ptr));
            let searcher_handle = SearcherHandle { ptr: UnsafeCell::new(searcher_ptr) };

            let builder = thread::Builder::new()
                .name("SearcherHandle".to_owned());

            let handle = builder.spawn_unchecked(
                move || 
                {
                    let s_handle = searcher_handle;
                    let thread = & mut ** s_handle.ptr.get();
                    log::debug!("Dereferenced search pointer.");
                    thread.cond_variable.lock();
                    thread.idle();
                }
            ).unwrap();

            self.handles.push(handle);
        }
    }

    ///
    /// Clears all threads.
    ///
    pub fn clear (& mut self)
    {
        self.threads.iter_mut()
            .map(|handle| unsafe { & mut ** (* handle).get() })
            .for_each(|thread| thread.clear());
    }

    ///
    /// Kills all threads.
    ///
    pub fn kill (& mut self) 
    {
        self.stop.store(true, Ordering::Relaxed);
        self.wait_for(SearcherEvent::Finish);

        let mut handles = Vec::with_capacity(self.threads.len());

        unsafe 
        {
            self.threads.iter()
                .map(|handle| & (** handle.get()))
                .for_each(|thread| { thread.kill.store(true, Ordering::SeqCst); });

            self.threads.iter()
                .map(|handle| & (** handle.get()))
                .for_each(|thread| { thread.cond_variable.set(); });

            while let Some(handle) = self.handles.pop()
            {
                handles.push(handle.join());
            }

            while let Some(handle) = self.threads.pop()
            {
                let thread : * mut Searcher = * handle.get();
                let pointer : NonNull<u8> = mem::transmute(NonNull::new_unchecked(thread));
                let layout = Layout::new::<Searcher>();
                dealloc(pointer.as_ptr(), layout);
            }
        }
    }

    ///
    /// Starts the search on the main thread, which has 
    /// the specific responsbility to collect the best 
    /// move in the position.
    ///
    pub fn launch (& mut self, state: & Board) 
    {
        log::info!("Search started on position '{}'.", state.notate());

        self.set_stop_requirement(false);

        self.cond.set();
        self.wait_for(SearcherEvent::Start);

        self.cond.lock();
        self.wait_for(SearcherEvent::Finish);

        self.set_stop_requirement(true);

        log::info!("Search ended on position '{}'.", state.notate());

        let mut best_move  = 0;
        let mut best_score = f32::NEG_INFINITY;

        self.threads.iter_mut()
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

        self.parent().best_move = best_move;

        self.print_move_table();
    }

    ///
    /// Creates a new thread pool and attaches the main thread.
    ///
    pub fn new () -> ThreadPool
    {
        let pool = ThreadPool 
        {
            mcts: ptr::null(),
            state: Board::blank(),

            threads: Vec::new(),
            handles: Vec::new(),

            cond: Arc::new(Latch::new()),

            stop: AtomicBool::new(true)
        };

        // Lock all conditions.

        pool.cond.lock();
    
        pool
    }

    ///
    /// Returns the MCTS instance behind this pool.
    ///
    pub fn parent (& mut self) -> & mut MCTS 
    {
        unsafe { mem::transmute(& self.mcts) }
    }

    ///
    /// Logs the move table formed by combining the roots of 
    /// each thread's move pool.
    ///
    pub fn print_move_table (& self)
    {
        let mut movemap : HashMap<MoveID, SearcherStats> = HashMap::new();
        for mv in & self.state.enumerate_moves()
        {
            let id : usize = mv.clone().into();
            movemap.insert(id, SearcherStats { tetromino: mv.notate(), visits: 0.0, prob: 0.0, eval: 0.0, components: 0 });
        }

        self.threads.iter()
            .map(|handle| unsafe { & (** handle.get()) })
            .for_each(
                |thread|
                {
                    for child in thread.children_of_immut(thread.root)
                    {
                        let key = child.in_action;
                        let mut entry = movemap.get_mut(& key).unwrap();

                        entry.visits += child.n;
                        entry.prob = ((entry.components as f32 * entry.prob) + child.p) / (entry.components as f32 + 1.0);
                        entry.eval = ((entry.components as f32 * entry.eval) + child.v) / (entry.components as f32 + 1.0);
                    }
                }
            );

        let mut movevec = movemap.into_values().into_iter().collect::<Vec<SearcherStats>>();
        movevec.sort_by(|a, b| std::primitive::f32::total_cmp(& a.eval, & b.eval));
        movevec.resize(20, SearcherStats { tetromino: "".to_owned(), eval: 0.0, prob: 0.0, visits: 0.0, components: 0 });

        let total_sims : usize = self.threads.iter()
            .map(|handle| unsafe { & (** handle.get()) })
            .map(|thread| thread.num_sims)
            .sum();

        log::info!("MCTS eval table ({} simulations) for '{}':\n{}", total_sims, self.state.notate(), Table::new(movevec).with(tabled::Style::psql()).to_string());
    }


    ///
    /// Unsafely sets the number of threads.
    ///
    pub fn set_num_threads (& mut self, num: usize)
    {
        if num > 0 
        {
            self.wait_for(SearcherEvent::Finish);
            self.kill();
            for _ in 0 .. num 
            {
                self.attach_one();
            }
        }
    }

    ///
    /// Sets the parent.
    ///
    pub fn set_parent (& mut self, mcts: & MCTS)
    {
        self.mcts = mcts;
    }

    ///
    /// Sets the stop state on the thread pool.
    ///
    pub fn set_stop_requirement (& mut self, to: bool)
    {
        self.stop.store(to, Ordering::Relaxed);
    }

    ///
    /// Waits for the conditions of every thread matching the filter 
    /// to evaluate to the same value.
    ///
    pub fn wait_for (& mut self, event: SearcherEvent)
    {
        self.threads.iter()
            .map(|handle| unsafe { & (** handle.get()) })
            .for_each(|thread| { thread.search_status.wait(event.clone().into()); });
    }
}
