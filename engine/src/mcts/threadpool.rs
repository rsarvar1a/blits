
#![allow(mutable_transmutes)]

use crate::config::*;
use crate::neural::network::Network;

use lits::{Board, Tetromino};

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

use super::node::{Outcome, MoveID};
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
#[derive(Debug)]
pub struct ThreadPool
{
    pub config: Config,

    pub state: Board,
    pub best_move: MoveID,

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
    pub fn attach_one (& mut self, policy: & Network)
    {
        unsafe 
        {
            let mem_layout = Layout::new::<Searcher>();
            let result = alloc_zeroed(mem_layout);
            let searcher_ptr : * mut Searcher = result.cast() as * mut Searcher;

            let searcher_id = self.threads.len();
            let cond_variable = self.cond.clone();
            let pool : * mut ThreadPool = self;

            ptr::write(searcher_ptr, Searcher::new(pool, self.config.clone(), policy, searcher_id, cond_variable));

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

        thread::sleep(std::time::Duration::from_millis(self.config.mcts.max_time_ms as u64));
        self.set_stop_requirement(true);

        self.cond.lock();
        self.wait_for(SearcherEvent::Finish);

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

                        if ! child.is_unsolved()
                        {
                            entry.visits += child.n;
                            entry.prob = ((entry.components as f32 * entry.prob) + child.p) / (entry.components as f32 + 1.0);
                            entry.components += 1;

                            entry.eval = match child.outcome.unwrap() 
                            {
                                Outcome::Win  => f32::INFINITY,
                                Outcome::Loss => f32::NEG_INFINITY
                            };
                        }
                        else 
                        {
                            entry.visits += child.n;
                            entry.prob = ((entry.components as f32 * entry.prob) + child.p) / (entry.components as f32 + 1.0);
                            entry.eval = ((entry.components as f32 * entry.eval) - child.v) / (entry.components as f32 + 1.0);
                            entry.components += 1;
                        }
                    }
                }
            );

        let mut movevec = movemap.into_values().into_iter().collect::<Vec<SearcherStats>>();
        movevec.sort_by(|a, b| std::primitive::f32::total_cmp(& b.eval, & a.eval));

        self.best_move = Tetromino::parse(& movevec.first().unwrap().tetromino).unwrap().into();
        self.print_move_table(& movevec);

        log::info!("Search ended on position '{}'.", state.notate());
    }

    ///
    /// Creates a new thread pool and attaches the main thread.
    ///
    pub fn new (config: & Config) -> ThreadPool
    {
        let pool = ThreadPool 
        {
            config: config.clone(),
            state: Board::blank(),
            best_move: 0,

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
    /// Logs the move table formed by combining the roots of 
    /// each thread's move pool.
    ///
    pub fn print_move_table (& self, movevec: & Vec<SearcherStats>)
    {
        let mut movevec = movevec.clone();
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
    pub fn set_num_threads (& mut self, num: usize, policy: & Network)
    {
        if num > 0 
        {
            self.wait_for(SearcherEvent::Finish);
            self.kill();
            for _ in 0 .. num 
            {
                self.attach_one(policy);
            }
        }
    }

    ///
    /// Sets the stop state on the thread pool.
    ///
    pub fn set_stop_requirement (& mut self, to: bool)
    {
        self.stop.store(to, Ordering::SeqCst);
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
