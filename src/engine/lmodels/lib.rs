

use bevy::prelude::*;
use linfa::prelude::*;

use linfa_kernel::{Kernel, KernelMethod};
use linfa_nn::{BallTreeIndex, KdTreeIndex,LinearSearchIndex, CommonNearestNeighbour, NearestNeighbour};


pub struct DataSet<T>{
    pub name: String,
    pub data: T ,
    kernal_method: KernalMethod,
    cnn_algo: CommonNearestNeighbour,
    nn_algo: NearestNeighbour

}


impl<T> DataSet<T>{
pub fn new(name: String, data: T) -> DataSet<T>{
    DataSet{
        name,
        data
    };}
}

impl<T> DataSet<T> {
    pub fn new(name: String, data: T) -> Self {
        DataSet {
            name,
            data,
            kernel_method: KernelMethod::Linear, // Default value
            nn_algo: CommonNearestNeighbour::KdTree(KdTreeIndex::new(data)),
        }
    }
}
trait Kerrnaltrait {
    type kernal_type; 

    

    //Polynomial, Linear, Gaussian, etc.
    fn set_nearest_neighbour(&self) -> KernelMethod;
    fn set_kernal_method(&mut self, nn: CommonNearestNeighbour);
    fn return_kernal(&self) -> self::kernal_type;

}


impl Kerrnaltrait for DataSet<vec<f64>> {
    type kernal_type = Kernel<f64>;
    type KernelType = Kernel<f64>;

    fn set_kernel_method(&mut self, method: KernelMethod) {
        self.kernel_method = method;
    }

    fn set_common_nearest_neighbour(&mut self, cnn: CommonNearestNeighbour) {
        self.cnn_algo = cnn;
    }

    fn set_nearest_neighbour(&mut self, nn: NearestNeighbour) {
        self.cnn_algo = nn;
    }

    fn build_kernel(&self) -> Self::KernelType {
        // Implementation to build and return the kernel
        Kernel::params()
        .method(self.kernel_method)
        .nearest_neighbour(self.cnn_algo)
    }

}



impl KerrnalSystem {
    pub fn new() -> Self {
        Self {}
    }
    fn process(&mut self, _time: f32, _delta: f32, _kernel: KernelMethod)
    {   self.kernel_method = KernelMethod::None;
        }
}

