
use crate::config::*;

use lits::board::Board;
use lits::outcome::Outcome;
use lits::tetromino::{Tetromino, TETROMINO_RANGE};

use super::input::*;
use super::memory::*;

use tch::{Device, IndexOp, Tensor};
use tch::jit::{IValue, TrainableCModule};
use tch::nn::{OptimizerConfig, Sgd, VarStore};

use utils::error::{error, Context, Result};

///
/// A network that functions simultaneously as a policy and state head.
///
/// The input shape is a [10, 10, 5] board image tensor; each column of 
/// 5 values is a one-hot encoding of the colour followed by the player 
/// tile value, which is non-zero if and only if the colour is None.
///
/// The policy output shape is a [1293] tetromino vector representing the 
/// relative (softmaxed) strength of each move according to the policy.
/// Illegal moves are not pre-masked in the output and must be handled 
/// by the caller.
///
/// The value output shape is a [1] value ranging from -1.0 to 1.0,
/// representing the network's prediction of the next state's favour 
/// in X's perspective.
///
#[derive(Debug)]
pub struct Network 
{
    config: NeuralConfig,
    vs: VarStore,
    model: TrainableCModule,
    mem: Vec<Memory>
}

impl Network 
{
    ///
    /// Returns the best tetromino in this position.
    ///
    pub fn argmax (& self, board: & Board) -> Tetromino 
    {
        let (policy, _values) = self.predict(board);
        let tensor = Tensor::of_slice(& policy);
        let argmax = tensor.argmax(0, false).i(0); 
        let mut indices : [i32; 1] = [0; 1];
        argmax.copy_data(& mut indices, 1);
        
        return Tetromino::from(indices[0] as usize);
    }

    ///
    /// For a given input tensor of board images, predicts the policy-value pairs.
    ///
    pub fn forward (& self, input: & Tensor) -> (Tensor, Tensor)
    {
        let mut copy = Tensor::new();
        copy.copy_(input);

        let input_tensor = IValue::Tensor(copy);
        let ivalue_tuple = self.model.forward_is(& [input_tensor]).unwrap();

        let mut policy = Tensor::new();
        let mut values = Tensor::new();

        match ivalue_tuple 
        {
            IValue::Tuple(tensor_vec) => 
            {
                if let IValue::Tensor ( tensor ) = tensor_vec.get(0).unwrap()
                {
                    policy = tensor.copy();
                }
                if let IValue::Tensor ( tensor ) = tensor_vec.get(1).unwrap()
                {
                    values = tensor.copy();
                }
            },
            _ => panic!("Failed to unroll forward from model.")
        };

        (policy, values)
    }

    ///
    /// Creates a network by loading an artifact file.
    ///
    pub fn from_artifact (config: & NeuralConfig, artifact: & str) -> Result<Network>
    {
        let mut net = Network::from_template(config)?;
        
        let artifact_path = format!("{}/trained/{}", config.path, artifact);
        net.vs.load(& artifact_path).context(format!("Failed to load weights file from '{}'.", & artifact_path))?;

        Ok(net)
    }

    pub fn from_best (config: & NeuralConfig) -> Result<Network>
    {
        Network::from_artifact(config, & config.best)
    }

    ///
    /// Creates a brand-new network from the template file.
    ///
    pub fn from_template (config: & NeuralConfig) -> Result<Network> 
    {
        let vs = VarStore::new(Device::cuda_if_available());
        let mem = vec![];
        let template_path = format!("{}/{}", config.path, config.template);
        let model = tch::TrainableCModule::load(& template_path, vs.root()).context(format!("Failed to load template file from '{}'.", & template_path))?;

        let mut net = Network { config: config.clone(), vs, model, mem };
        net.model.set_eval();

        Ok(net)
    }

    ///
    /// Given an input board, returns the policy vector and a value estimation.
    ///
    pub fn predict (& self, board: & Board) -> ([f32; TETROMINO_RANGE], f32)
    {
        let input : Tensor = Input::from(board.clone()).0;
        let (policy, values) = self.forward(& input);

        // Extract the policy data by masking it against the set of valid 
        // moves in this state.

        let mut mask : [f32; TETROMINO_RANGE] = [0.0; TETROMINO_RANGE];
        for tetromino in board.enumerate_moves()
        {
            let idx = <lits::Tetromino as Into<usize>>::into(tetromino.clone());
            mask[idx] = 1.0;
        }
        
        let mut policy_data = [0.0; TETROMINO_RANGE];
        policy.copy_data::<f32>(& mut policy_data, TETROMINO_RANGE);

        for i in 0 .. TETROMINO_RANGE 
        {
            policy_data[i] *= mask[i];
        }

        // Extract the value prediction. 
        
        let mut value_data = [0.0; 1];
        values.copy_data::<f32>(& mut value_data, 1);
        let value = value_data[0];

        (policy_data as [f32; TETROMINO_RANGE], value)
    }

    ///
    /// Constructs and remembers a memory. The memory is stored in terms 
    /// of the moving player's perspective. In other words, the input 
    /// tensor sets player tiles of that player to 1 and opposing tiles to 
    /// -1, and the end result is 1 if and only if the optimizing player 
    /// won the game.
    ///
    pub fn remember (& mut self, board: & Board, result: & Outcome)
    {
        let state = Input::from(board.clone()).0;

        let mut mask = [0.0; TETROMINO_RANGE];
        board.enumerate_moves().iter().for_each(|t| { mask[<Tetromino as Into::<usize>>::into(t.clone())] = 1.0; } );
        let policy_valid = Tensor::of_slice(& mask);

        let val = match result 
        {
            Outcome::X (_) => 1.0,
            Outcome::O (_) => -1.0,
            _              => 0.0,
        };
        let end_result = Tensor::of_slice(& [val]) * board.to_move().value();

        let memory = Memory { state, policy_valid, end_result };
        self.mem.push(memory);
    }

    ///
    /// Saves this model's weights.
    ///
    pub fn save (& self, group: & str, path: & str) -> Result<()> 
    {
        let artifact_path = format!("{}/trained/{}/{}", self.config.path, group, path);
        self.vs.save(& artifact_path).context(error!(format!("Failed to save model to path '{}'.", & artifact_path)))?;
        Ok(())
    }

    ///
    /// Trains this model on the given batch tensors of memory components.
    ///
    pub fn train (& mut self)
    {
        self.model.set_train();

        let mut optimizer = Sgd::default().build(& self.vs, self.config.learning_rate as f64).unwrap();

        for _epoch in 1 ..= self.config.epochs 
        {
            for mem in & self.mem 
            {
                let (policy, values) = self.forward(& mem.state);

                let loss_policy = policy.cross_entropy_for_logits(& mem.policy_valid).sum(tch::Kind::Float);
                let loss_values = (values - & mem.end_result).pow_tensor_scalar(self.config.exp as f64);
                optimizer.backward_step(& (& loss_policy + & loss_values));
            }
        }

        self.model.set_eval();
    }
}
