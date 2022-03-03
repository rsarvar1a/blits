
use lits::Board;

use tch::Tensor;

///
/// Represents a core memory of (si, pi, z0).
///
/// The policy is trained against the mask, and the value is 
/// trained against the end result of the game.
///
#[derive(Debug)]
pub struct Memory 
{
    pub board: Board,
    pub policy_valid: Tensor,
    pub end_result: Tensor
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

