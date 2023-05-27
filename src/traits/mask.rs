

pub trait MaskOperations {
    fn size(&self) -> Vec2;
    fn fits_in_mask(&self, other_mask: &Self, position: Vec2) -> bool;
    fn is_valid_placement(
        &self,
        position: Vec2,
        world: &World,
        valid_tile_types: &[TileType],
        valid_mask: Option<&Self>,
    ) -> bool;
}


impl MaskOperations for Vec<Vec<u8>> {
    fn size(&self) -> Vec2 {
        Vec2::new(self[0].len() as f32, self.len() as f32)
    }


    fn fits_in_mask(&self, other_mask: &Self, position: Vec2) -> bool {

           let other_size = other_mask.size();
       }





       fn is_valid_placement(
       &self,
       position: Vec2,
       world: &World,
       valid_tile_types: &[TileType],
       valid_mask: Option<&Self>,
   ) -> bool {

       for y in 0..self.len() {
           for x in 0..self[0].len() {

       if let Some(valid_mask) = valid_mask {
                   if !self.fits_in_mask(valid_mask, Vec2::new(x as f32, y as f32)) {
                       return false;
                   }
               }

           }
       }// end FORZ!
       true 
   }
}
