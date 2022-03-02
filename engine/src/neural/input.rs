
use lits::*;

use tch::{IndexOp, Tensor};

///
/// Represents an input representation converting from a board position 
/// to a neural network input shape. The shape of an input tensor is 
/// a 10 x 10 x 5 shape; for each tile there is a one-hot encoding of 
/// the colour, and then a value encoding of the player's tile. The 
/// encoding is produced in the perspective of the player to move.
///
pub struct Input (pub Tensor);

impl std::convert::From<Board> for Input 
{
    fn from (board: Board) -> Input 
    {
        let mut tensor = Tensor::of_slice::<f32>(& [0.0; 500]);
        tensor = tensor.reshape(& [5, 10, 10]);

        for i in 0 .. 10 
        {
            for j in 0 .. 10
            {
                let i = i as i32;
                let j = j as i32;

                for colour in [Colour::L, Colour::I, Colour::T, Colour::S]
                {
                    let c = colour.as_index() as i32;
                    let val : f32 = match board.colour_at(i, j) == colour 
                    {
                        true  => 1.0,
                        false => 0.0
                    };

                    tensor.i((c as i64, i as i64, j as i64)).copy_(& Tensor::of_slice(& [val]));
                }

                if board.colour_at(i, j) == Colour::None 
                {
                    // The value is from the current player's perspective. Multiplying the player 
                    // at the tile by the player to move assures that player <#> is in <#>'s
                    // perspective and the other player is represented by -1s.

                    let pval : f32 = (board.player_at(i, j).value() * board.to_move().value()) as f32;
                    tensor.i((4, i as i64, j as i64)).copy_(& Tensor::of_slice(& [pval]));
                }
            }
        }

        Input(tensor)
    }
}

