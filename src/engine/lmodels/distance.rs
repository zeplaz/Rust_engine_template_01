use linfa_clustering::Dbscan;
use linfa_nn::{Distance, CommonNearestNeighbour};

// This allows the CustomDist type to be used as a distance function
impl Distance<f64> for CustomDist {
 
}



fn disntance_system () {
    let params = Dbscan::params_with(min_points, CustomDist, CommonNearestNeighbour::KdTree);
    let cluster_memberships = params.tolerance(tol).transform(dataset).unwrap();
}

