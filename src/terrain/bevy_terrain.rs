// TerrainType enum
#[derive(Debug, Clone)]
#[doc(alais = "Paramter_labels")]
pub enum TerrainType {
    Grass,
    Forest,
    Swamp,
    Water,
    Cliff,
    Concrete,
    Sand,
    Dirt,
    Snow,
    Stone,
}

// infsastucure, or added other fetures.,, mbooling bitwize ones,
pub struct Terrain_Features {
    road: bool,
    track: bool,
}

// should be floats, "envoment' of intraction";; this should be influnced by infastuctree
//or features..
pub struct terrain_environment {
    ambent_tempature: Vec2,
    roughness: f64,
    moisture: f64,
    elevation: f64,
    sunlight: f64,
    cloud_coverage: f64,
    water_desity: f64,
    water_salinity: f64,
}

//tags of flora
pub enum terrain_tags {
    Setaphata,
    moss,
    vines,
    shrubs,
    broad_leafed_trees,
    coniferous_trees,
    grass,
}

pub enum Ceres_subtags {
    Cereal,
    Legumes,
}
//-> collection of images, of their kind->blend for new vierities.
 pub enum flower_subtags {
     dafidals,
     forgetmenoghts,
     bella,
     dandlions,
     blue iris,
     a... 
 }
