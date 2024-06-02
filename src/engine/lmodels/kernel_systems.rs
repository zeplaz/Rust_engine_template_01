


use bevy::prelude::*;
use linfa::prelude::*;

use linfa_kernel::{Kernel, KernelMethod};
use linfa_nn::{BallTreeIndex, CommonNearestNeighbour, NearestNeighbour};


use lib::DataSet;

fn initialize_kernel_with_ball_tree(dataset:DataSet) -> Kernel<f64> {
    // Assuming `dataset` is your data for the terrain system
    //let dataset = /* your dataset initialization */
    
    // Initialize BallTreeIndex with your dataset
    let ball_tree = BallTreeIndex::new(&dataset.data);

    // Wrap BallTreeIndex in CommonNearestNeighbour
    let nn_algo = CommonNearestNeighbour::BallTree(ball_tree);

    // Initialize Kernel with BallTreeIndex
    Kernel::params().method(KernelMethod::Polynomial).nearest_neighbour(nn_algo)
}

fn terrain_analysis(kernel: &Kernel<f64>) {
    // Use kernel for terrain analysis
    // Ensure compatibility with BallTreeIndex
}



impl KerrnalSystem {
    pub fn new() -> Self {
        Self {}
    }
    fn process(&mut self, _time: f32, _delta: f32, _kernel: KernelMethod)
    {   self.kernel_method = KernelMethod::None;
        }
}








/// including pathfinding

impl PathfindingSystem {
    fn find_path(&self, path_data: Dataset) {
        let kernel = Kernel::params().method(KernelMethod::Polynomial).transform(&path_data);
        // Use kernel methods for path decisions
    }
}
