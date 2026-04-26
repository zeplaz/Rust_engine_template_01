use bevy::prelude::*;
use rand::Rng;

// Assuming EntityId is defined elsewhere in the codebase
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(u32);

impl EntityId {
    pub fn from_u32(value: u32) -> Self {
        Self(value)
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

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
        sites.push(VoronoiSite { 
            id: EntityId::from_u32(i), 
            position 
        });
    }

    sites
}

pub struct RegularVoronoi;

impl CalculateDistance for RegularVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        a.position.distance(b.position)
    }
}

// Implementation for regular Voronoi diagram generation
pub fn voronoi_diagram_generation(num_sites: u32, width: u32, height: u32) -> Vec<Vec<VoronoiSite>> {
    let regular_voronoi = RegularVoronoi;
    let sites = generate_sites(num_sites, width as f32, height as f32);
    assign_points_to_sites(&regular_voronoi, &sites, width, height)
}

// Generic point assignment function to avoid duplication
fn assign_points_to_sites<T: CalculateDistance>(
    distance_calculator: &T,
    sites: &[VoronoiSite], 
    width: u32, 
    height: u32
) -> Vec<Vec<VoronoiSite>> {
    let mut regions: Vec<Vec<VoronoiSite>> = vec![vec![]; sites.len()];

    for y in 0..height {
        for x in 0..width {
            let current_point = Vec2::new(x as f32, y as f32);
            let mut closest_site = &sites[0];
            let mut min_distance = distance_calculator.distance(
                closest_site, 
                &VoronoiSite { id: EntityId::from_u32(0), position: current_point }
            );

            for site in sites.iter().skip(1) {
                let distance = distance_calculator.distance(
                    site, 
                    &VoronoiSite { id: EntityId::from_u32(0), position: current_point }
                );
                if distance < min_distance {
                    min_distance = distance;
                    closest_site = site;
                }
            }

            let site_id_as_u32 = closest_site.id.as_u32();
            regions[site_id_as_u32 as usize].push(VoronoiSite { 
                id: EntityId::from_u32(0), 
                position: current_point 
            });
        }
    }

    regions
}

// Centroidal Voronoi implementation
pub fn centroidal_voronoi_diagram_generation(num_sites: u32, width: u32, height: u32, iterations: u32) -> Vec<Vec<VoronoiSite>> {
    let mut sites = generate_sites(num_sites, width as f32, height as f32);
    let regular_voronoi = RegularVoronoi;
    
    // Lloyd's algorithm for relaxation
    for _ in 0..iterations {
        // Assign points to sites
        let regions = assign_points_to_sites(&regular_voronoi, &sites, width, height);
        
        // Calculate new centers
        for (i, region) in regions.iter().enumerate() {
            if !region.is_empty() {
                let mut center = Vec2::ZERO;
                for point in region {
                    center += point.position;
                }
                center /= region.len() as f32;
                
                // Update site position to centroid
                sites[i].position = center;
            }
        }
    }
    
    // Final assignment
    assign_points_to_sites(&regular_voronoi, &sites, width, height)
}

// Weighted Voronoi implementation
pub struct WeightedVoronoiSite {
    pub site: VoronoiSite,
    pub weight: f32,
}

pub struct AdditivelyWeightedVoronoi {
    pub site_weights: Vec<f32>,
}

impl CalculateDistance for AdditivelyWeightedVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        let base_distance = a.position.distance(b.position);
        
        // Apply weight if it's a site we have a weight for
        if a.id.as_u32() < self.site_weights.len() as u32 {
            let weight = self.site_weights[a.id.as_u32() as usize];
            base_distance - weight
        } else {
            base_distance
        }
    }
}

pub fn additively_weighted_voronoi_diagram_generation(
    num_sites: u32, 
    width: u32, 
    height: u32,
    weights: Option<Vec<f32>>
) -> Vec<Vec<VoronoiSite>> {
    let sites = generate_sites(num_sites, width as f32, height as f32);
    
    // Create weights if not provided
    let site_weights = match weights {
        Some(w) => w,
        None => {
            let mut rng = rand::thread_rng();
            (0..num_sites).map(|_| rng.gen_range(0.0..10.0)).collect()
        }
    };
    
    let weighted_voronoi = AdditivelyWeightedVoronoi { site_weights };
    assign_points_to_sites(&weighted_voronoi, &sites, width, height)
}

// Power Voronoi (multiplicative weights)
pub struct PowerVoronoi {
    pub site_weights: Vec<f32>,
}

impl CalculateDistance for PowerVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        let base_distance = a.position.distance_squared(b.position);
        
        // Apply weight if it's a site we have a weight for
        if a.id.as_u32() < self.site_weights.len() as u32 {
            let weight = self.site_weights[a.id.as_u32() as usize];
            base_distance - weight * weight
        } else {
            base_distance
        }
    }
}

pub fn power_voronoi_diagram_generation(
    num_sites: u32, 
    width: u32, 
    height: u32,
    weights: Option<Vec<f32>>
) -> Vec<Vec<VoronoiSite>> {
    let sites = generate_sites(num_sites, width as f32, height as f32);
    
    // Create weights if not provided
    let site_weights = match weights {
        Some(w) => w,
        None => {
            let mut rng = rand::thread_rng();
            (0..num_sites).map(|_| rng.gen_range(0.0..5.0)).collect()
        }
    };
    
    let power_voronoi = PowerVoronoi { site_weights };
    assign_points_to_sites(&power_voronoi, &sites, width, height)
}

// Circular Voronoi (cones instead of planes)
pub struct CircularVoronoi {
    pub height_scale: f32,
}

impl CalculateDistance for CircularVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        let base_distance = a.position.distance(b.position);
        // Use distance to create a cone-based voronoi
        base_distance * self.height_scale
    }
}

pub fn circular_voronoi_diagram_generation(
    num_sites: u32, 
    width: u32, 
    height: u32,
    height_scale: f32
) -> Vec<Vec<VoronoiSite>> {
    let sites = generate_sites(num_sites, width as f32, height as f32);
    let circular_voronoi = CircularVoronoi { height_scale };
    assign_points_to_sites(&circular_voronoi, &sites, width, height)
}

// Manhattan Voronoi
pub struct ManhattanVoronoi;

impl CalculateDistance for ManhattanVoronoi {
    fn distance(&self, a: &VoronoiSite, b: &VoronoiSite) -> f32 {
        (a.position.x - b.position.x).abs() + (a.position.y - b.position.y).abs()
    }
}

pub fn manhattan_voronoi_diagram_generation(num_sites: u32, width: u32, height: u32) -> Vec<Vec<VoronoiSite>> {
    let sites = generate_sites(num_sites, width as f32, height as f32);
    let manhattan_voronoi = ManhattanVoronoi;
    assign_points_to_sites(&manhattan_voronoi, &sites, width, height)
}

// Generate Voronoi region boundaries
pub fn generate_voronoi_boundaries(regions: &[Vec<VoronoiSite>], width: u32, height: u32) -> Vec<Vec<Vec2>> {
    let mut boundaries = Vec::with_capacity(regions.len());
    
    // For each region
    for region_points in regions {
        if region_points.is_empty() {
            boundaries.push(Vec::new());
            continue;
        }
        
        // Create a grid to track region membership
        let mut grid = vec![vec![false; width as usize]; height as usize];
        
        // Mark all cells in this region
        for point in region_points {
            let x = point.position.x as usize;
            let y = point.position.y as usize;
            if x < width as usize && y < height as usize {
                grid[y][x] = true;
            }
        }
        
        // Find boundary points
        let mut boundary_points = Vec::new();
        
        // Define the 8 neighbor directions
        let directions = [
            (-1, -1), (0, -1), (1, -1),
            (-1, 0),           (1, 0),
            (-1, 1),  (0, 1),  (1, 1)
        ];
        
        for y in 0..height as usize {
            for x in 0..width as usize {
                if grid[y][x] {
                    // Check if this is a boundary point
                    let mut is_boundary = false;
                    
                    for (dx, dy) in &directions {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            if !grid[ny as usize][nx as usize] {
                                is_boundary = true;
                                break;
                            }
                        } else {
                            // Points at the edge of the grid are always boundary points
                            is_boundary = true;
                            break;
                        }
                    }
                    
                    if is_boundary {
                        boundary_points.push(Vec2::new(x as f32, y as f32));
                    }
                }
            }
        }
        
        boundaries.push(boundary_points);
    }
    
    boundaries
}