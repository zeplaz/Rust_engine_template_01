<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.1" name="rails" tilewidth="64" tileheight="64" tilecount="55" columns="11">
 <image source="../unsorted_asset_tile_maps/railroad_track.png" width="704" height="320"/>
 <wangsets>
  <wangset name="rail_01" type="corner" tile="-1">
   <wangcolor name="off_rail" color="#ff0000" tile="-1" probability="1"/>
   <wangcolor name="on_rail" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,0,0,1,0,0,0,1"/>
   <wangtile tileid="1" wangid="0,1,0,0,0,0,0,1"/>
   <wangtile tileid="11" wangid="0,1,0,0,0,0,0,1"/>
  </wangset>
  <wangset name="railz" type="edge" tile="-1">
   <wangcolor name="off_rail" color="#ff0000" tile="-1" probability="1"/>
   <wangcolor name="on_rail" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="9" wangid="0,0,2,0,1,0,2,0"/>
   <wangtile tileid="10" wangid="1,0,2,0,0,0,2,0"/>
  </wangset>
  <wangset name="rail" type="mixed" tile="-1">
   <wangcolor name="off_rail" color="#ff0000" tile="-1" probability="1"/>
   <wangcolor name="on_rail" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="1,1,2,1,2,2,1,1"/>
   <wangtile tileid="1" wangid="1,1,1,2,2,1,2,1"/>
   <wangtile tileid="2" wangid="1,1,2,2,2,2,2,1"/>
   <wangtile tileid="22" wangid="1,1,1,0,0,0,1,1"/>
   <wangtile tileid="23" wangid="1,1,1,0,0,0,1,1"/>
   <wangtile tileid="24" wangid="1,1,1,0,0,0,1,1"/>
   <wangtile tileid="25" wangid="1,1,1,0,0,0,1,1"/>
   <wangtile tileid="26" wangid="1,1,1,0,0,0,1,1"/>
   <wangtile tileid="33" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="34" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="35" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="36" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="37" wangid="1,0,0,0,0,0,0,1"/>
   <wangtile tileid="44" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="45" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="46" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="47" wangid="1,1,0,0,0,0,0,1"/>
   <wangtile tileid="48" wangid="1,1,0,0,0,0,0,1"/>
  </wangset>
 </wangsets>
</tileset>
