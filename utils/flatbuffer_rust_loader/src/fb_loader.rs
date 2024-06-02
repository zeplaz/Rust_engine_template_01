



extern crate flatbuffers; 

#[allow(dead_code, unused_imports)]
#[path = "./data/flatbuffers/fb_schema_test01.rs"]

mod schema_test01;

pub use schema_test01::entities::properties::{Strukturave, MineType,FactoryType,PowerType,ResidenceType, ApartmentUnitType, HighriseArgs, ApartmentsType};


#[allow(clippy::float_cmp)]
fn run_builder(){
    let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(8192);


    let sovet_tryplex_name = builder.create_string("Sovet Tryplex");

    let Highrise1 = Highrise::create(&mut builder, HighriseArgs{
        units_available: some(), // call get aptmneunit thing
    });

    let sovet_tryplex = Strukturave::create(&mut builder, &StrukturaveArgs{
        pos: Some(&Vec3::new(1.0f32, 2.0f32, 3.0f32)),
        name: Some(sovet_tryplex_name),
        ..Default::default()
    });
    builder.finish(sovet_tryplex, None);
}



