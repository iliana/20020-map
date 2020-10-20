Cesium.Camera.DEFAULT_VIEW_RECTANGLE = Cesium.Rectangle.fromDegrees(-124.848974, 24.396308, -66.885444, 49.384358);
Cesium.Camera.DEFAULT_VIEW_FACTOR = 0.1;

const viewer = new Cesium.Viewer('map', {
  animation: false,
  baseLayerPicker: false,
  geocoder: false,
  imageryProvider: new Cesium.ArcGisMapServerImageryProvider({
    url: 'https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer'
  }),
  infoBox: false,
  requestRenderMode: true,
  sceneModePicker: false,
  terrainProvider: new Cesium.ArcGISTiledElevationTerrainProvider({
    url: 'https://elevation3d.arcgis.com/arcgis/rest/services/WorldElevation3D/Terrain3D/ImageServer'
  }),
  timeline: false,
});
