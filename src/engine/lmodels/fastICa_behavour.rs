
use bevy::prelude::*;
use linfa::prelude::*;
use linfa_ica::FastIca;











// inclduing behavours


impl PlayerBehaviorSystem {
    fn analyze_behavior(&self, player_data: Dataset) {
        let ica = FastIca::new().fit(&player_data).expect("ICA fitting failed");
        // Interpret results to adapt NPC actions
    }}