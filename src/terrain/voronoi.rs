use super idgen::EntityId;


pub struct VoronoiSite {
    pub id: EntityId,
    pub position: Vec2,
}

pub trait CalculateDistance {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32;
}



fn generate_sites(num_sites: u32, width: f32, height: f32) -> Vec<VoronoiSite> {
    let mut rng = rand::thread_rng();
    let mut sites = Vec::new();

    for i in 0..num_sites {
        let position = Vec2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height));
        sites.push(VoronoiSite { id: i, position });
    }

    sites
}



use super::voronoi::{VoronoiSite, CalculateDistance};

pub struct RegularVoronoi;

impl CalculateDistance for RegularVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        a.position.distance(b.position)
    }
}

pub fn voronoi_diagram_generation() {
    // Implementation for regular Voronoi diagram generation
}



//use super::regular_voronoi::{RegularVoronoi, voronoi_diagram_generation};

pub fn centroidal_voronoi_diagram_generation() {
    // Implementation for centroidal Voronoi diagram generation
    voronoi_diagram_generation(); // Use the regular Voronoi diagram generation as a basis
}


//use super::voronoi::{VoronoiSite, CalculateDistance};

pub struct WeightedVoronoiSite {
    site: VoronoiSite,
    weight: f32,
}

pub struct AdditivelyWeightedVoronoi;

impl CalculateDistance for AdditivelyWeightedVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        a.position.distance(b.position) // Add the weight attribute to the distance calculation
    }
}

pub fn additively_weighted_voronoi_diagram_generation() {
    // Implementation for Additively Weighted Voronoi diagram generation
}



//use super::additively_weighted_voronoi::{WeightedVoronoiSite, additively_weighted_voronoi_diagram_generation};

pub fn power_voronoi_diagram_generation() {
    // Implementation for Power Voronoi diagram generation
    additively_weighted_voronoi_diagram_generation(); // Use the Additively Weighted Voronoi diagram generation as a basis
}


//use super::regular_voronoi::{RegularVoronoi, voronoi_diagram_generation};

pub fn circular_voronoi_diagram_generation() {
    // Implementation for Circular Voronoi diagram generation
    voronoi_diagram_generation(); // Use the regular Voronoi diagram generation as a basis
}

//use super::voronoi::{VoronoiSite, CalculateDistance};

pub struct ManhattanVoronoi;

fn assign_points_to_sites_manhattan(sites: &[VoronoiSite], width: u32, height: u32) -> Vec<Vec<VoronoiSite>> {
    let manhattan_voronoi = ManhattanVoronoi;
    let mut regions: Vec<Vec<VoronoiSite>> = vec![vec![]; sites.len()];

    for y in 0..height {
        for x in 0..width {
            let current_point = Vec2::new(x as f32, y as f32);
            let mut closest_site = &sites[0];
            let mut min_distance = manhattan_voronoi.distance(closest_site, &VoronoiSite { id: 0, position: current_point });

            for site in sites.iter().skip(1) {
                let distance = manhattan_voronoi.distance(site, &VoronoiSite { id: 0, position: current_point });
                if distance < min_distance {
                    min_distance = distance;
                    closest_site = site;
                }
            }

            regions[closest_site.id as usize].push(VoronoiSite { id: 0, position: current_point });
        }
    }

    regions
}

impl CalculateDistance for ManhattanVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        (a.position.x - b.position.x).abs() + (a.position.y - b.position.y).abs()
    }
}

pub fn manhattan_voronoi_diagram_generation(num_sites: u32, width: u32, height: u32) -> Vec<Vec<VoronoiSite>> {
    let sites = generate_sites(num_sites, width as f32, height as f32);
    let regions = assign_points_to_sites_manhattan(&sites, width, height);
    regions
}
